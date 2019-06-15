use chrono::Utc;
use crate::inserts::{NewEntity, NewStatus, NewTwitterUserID};
use crate::models::{Status, TwitterUser};
use crate::sources::{DeleteError, LoadError, StatusSource};
use crate::types::Source;
use diesel::prelude::*;
use egg_mode::tweet::{delete, unretweet, user_timeline};
use egg_mode::{user::{UserID, blocks_ids}, KeyPair, Token};
use futures::Stream;
use std::collections::HashMap;
use std::env;
use tokio::runtime::current_thread::block_on_all;

#[derive(Clone, Debug)]
pub struct Twitter {
    pub token: Token,
    pub id: UserID<'static>,
}

impl Twitter {
    pub fn source() -> Source {
        Source::Twitter
    }

    pub fn load() -> Result<Box<StatusSource>, LoadError> {
        let tw = Self::load_unboxed()?;
        Ok(Box::new(tw))
    }

    pub fn load_unboxed() -> Result<Self, LoadError> {
        let con_token = KeyPair::new(
            env::var("TWITTER_CONSUMER_KEY")?,
            env::var("TWITTER_CONSUMER_SECRET")?,
        );
        let access_token = KeyPair::new(
            env::var("TWITTER_ACCESS_TOKEN_KEY")?,
            env::var("TWITTER_ACCESS_TOKEN_SECRET")?,
        );
        let token = Token::Access {
            consumer: con_token,
            access: access_token,
        };

        let uid: u64 = env::var("TWITTER_USER_ID")?
            .parse()
            .expect("TWITTER_USER_ID must be u64");

        Ok(Self {
            token,
            id: uid.into(),
        })
    }

    fn latest_2_ids_in_db(conn: &PgConnection) -> (Option<u64>, Option<u64>) {
        use crate::models::{pg_repeat, pg_to_number};
        use crate::schema::statuses::dsl::*;

        let mut ids: Vec<u64> = statuses.select(source_id)
            .filter(source.eq(Source::Twitter))
            .filter(is_repost.eq(false))
            .filter(deleted_at.is_null())
            // Awful, but less awful than implementing the cast function:
            .order_by(pg_to_number(source_id, pg_repeat("9", 25)).desc())
            .limit(2)
            .load::<String>(conn)
            .expect("!! Can’t retrieve penultimate twitter source ID from db")
            .iter()
            .map(|sid| sid.parse::<u64>().expect("!! Can’t parse twitter source ID"))
            .collect();

        (ids.pop(), ids.pop())
        // penultimate, latest
    }

    // (fetched, inserted)
    pub fn fetch_block_ids(&self, conn: &PgConnection) -> (usize, usize) {
        let mut fetched = 0;
        let mut inserted = 0;

        use std::time::{Instant, Duration};
        let mut bagstart = Instant::now();
        let mut blockbag: Vec<NewTwitterUserID> = Vec::with_capacity(5000);
        let blocklist = blocks_ids(&self.token);
        block_on_all(blocklist.for_each(|id| {
            fetched += 1;

            blockbag.push((*id).into());

            if fetched % 4000 == 0 {
                println!("=> Fetched {} block IDs, saving...", fetched);

                let inserted_ids: Vec<TwitterUser> = {
                    use crate::schema::twitter_users::dsl::*;
                    diesel::insert_into(twitter_users)
                        .values(&blockbag)
                        .on_conflict(source_id)
                        .do_nothing()
                        .get_results(conn)
                        .expect("!! Failed to insert IDs in db")
                };

                inserted += inserted_ids.len();
                blockbag.clear();

                let time_left = 60 - bagstart.elapsed().as_secs();
                println!("-- Inserted {} new IDs so far, sleeping {} seconds", inserted, time_left);
                std::thread::sleep(Duration::from_secs(time_left));
                bagstart = Instant::now();
            }

            Ok(())
        })).expect("!! Can’t read twitter blocks");

        let inserted_ids: Vec<TwitterUser> = {
            use crate::schema::twitter_users::dsl::*;
            diesel::insert_into(twitter_users)
                .values(&blockbag)
                .on_conflict(source_id)
                .do_nothing()
                .get_results(conn)
                .expect("!! Failed to insert IDs in db")
        };

        inserted += inserted_ids.len();

        (fetched, inserted)
    }
}

impl StatusSource for Twitter {
    /*
    200
    --- <-- if latest is not in packet, cursor down next page
    200
    --- <-- etc
    200


    ???

    ---
    latest <-- included in thing
    penultimate <-- what we request with
    */

    fn sync(&self, conn: &PgConnection) -> bool {
        let (penultimate, latest) = Self::latest_2_ids_in_db(conn);
        let latest = latest.or(penultimate).unwrap_or(0);
        println!(":: Latest twitter ID we have:\t\t{}", latest);
        if penultimate.is_some() {
            println!(
                ":: Penultimate twitter ID we have:\t{}",
                penultimate.unwrap()
            );
        }

        let mut statusbag: Vec<NewStatus> = vec![];
        let mut entitybag: HashMap<String, Vec<NewEntity>> = HashMap::new();
        let mut timeline = user_timeline(self.id, true, true, &self.token).with_page_size(200);
        let mut batch = 0;

        loop {
            // Get tweets older than penultimate, which should include the *latest*
            let (tl, feed) = block_on_all(timeline.older(penultimate))
                .expect("!! Can’t read twitter timeline");
            batch += 1;
            timeline = tl;
            let mut contains_latest = false;
            let mut ntweets = 0;
            for tweet in &feed {
                ntweets += 1;
                statusbag.push((*tweet).into());

                if let Some(ref ents) = tweet.extended_entities {
                    entitybag.insert(format!("{}", tweet.id), NewEntity::from_extended(ents));
                }

                if tweet.id == latest {
                    contains_latest = true;
                }
            }

            println!(
                "-> Batch {} ({} tweets) contains latest? {}",
                batch, ntweets, contains_latest
            );

            if contains_latest || ntweets == 0 || statusbag.len() >= 3200 {
                break;
            }
        }

        statusbag.reverse();

        println!(
            "=> Made {} calls to twitter and retrieved {} tweets",
            batch,
            statusbag.len()
        );

        use diesel::insert_into;

        let inserted_tweets: Vec<Status> = {
            use crate::schema::statuses::dsl::*;
            insert_into(statuses)
                .values(&statusbag)
                .on_conflict(source_id)
                .do_nothing()
                .get_results(conn)
                .expect("!! Failed to insert tweets in db")
        };

        let inserted_len = inserted_tweets.len();

        let mut entitysack = Vec::with_capacity(entitybag.len() * 4);
        for inserted in &inserted_tweets {
            if let Some(ents) = entitybag.remove(&inserted.source_id) {
                for mut ent in ents.into_iter() {
                    ent.status_id = inserted.id;
                    entitysack.push(ent);
                }
            }
        }

        entitysack.reverse();

        let entitied = {
            use crate::schema::entities::dsl::*;
            insert_into(entities)
                .values(&entitysack)
                .on_conflict(source_id)
                .do_nothing()
                .execute(conn)
                .expect("!! Failed to insert entity metadata in db")
        };

        let hint = if inserted_len == statusbag.len() - 1 {
            "as expected"
        } else if inserted_len >= statusbag.len() {
            "something’s odd" // likely some tweet(s) deleted from timeline directly
        } else {
            "some duplicates"
        };

        println!(
            "=> Inserted {} new tweets in DB ({}) and {} entities",
            inserted_len, hint, entitied
        );

        true
    }

    fn delete(&self, conn: &PgConnection, status: &Status) -> Result<(), DeleteError> {
        if status.deleted_at.is_some() {
            return Err(DeleteError::AlreadyDone);
        }
        if status.source != Source::Twitter {
            return Err(DeleteError::WrongSource);
        }

        let id: u64 = status
            .source_id
            .parse()
            .expect("!! Can’t parse source ID");

        block_on_all(if status.is_repost {
            unretweet(id, &self.token)
        } else {
            delete(id, &self.token)
        })?;

        {
            use crate::schema::statuses::dsl::*;
            diesel::update(statuses.find(status.id))
                .set(deleted_at.eq(Utc::now()))
                .execute(conn)?;
        }

        Ok(())
    }
}
