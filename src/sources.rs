use diesel::prelude::*;
use egg_mode::{tweet::user_timeline, user::UserID, KeyPair, Token};
use crate::inserts::NewStatus;
use std::env;
use tokio::runtime::current_thread::block_on_all;

macro_rules! expect_env {
    ($name:expr) => {
        env::var($name).expect(&format!("{} must be set", $name))
    };
}

pub struct Twitter {
    token: Token,
    id: UserID<'static>,
}

impl Twitter {
    pub fn load() -> Self {
        let con_token = KeyPair::new(
            expect_env!("TWITTER_CONSUMER_KEY"),
            expect_env!("TWITTER_CONSUMER_SECRET"),
        );
        let access_token = KeyPair::new(
            expect_env!("TWITTER_ACCESS_TOKEN_KEY"),
            expect_env!("TWITTER_ACCESS_TOKEN_SECRET"),
        );
        let token = Token::Access {
            consumer: con_token,
            access: access_token,
        };

        let uid: u64 = expect_env!("TWITTER_USER_ID")
            .parse()
            .expect("TWITTER_USER_ID must be u64");

        Self {
            token,
            id: uid.into(),
        }
    }

    fn latest_2_ids_in_db(conn: &PgConnection) -> (Option<u64>, Option<u64>) {
        use crate::models::{pg_repeat, pg_to_number};
        use crate::schema::statuses::dsl::*;
        use crate::types::Source;

        let mut ids: Vec<u64> = statuses.select(source_id)
            .filter(source.eq(Source::Twitter))
            .filter(is_repost.eq(false))
            // Awful, but less awful than implementing the cast function:
            .order_by(pg_to_number(source_id, pg_repeat("9", 25)).desc())
            .limit(2)
            .load::<String>(conn)
            .expect("Can’t retrieve penultimate twitter source ID from db")
            .iter()
            .map(|sid| sid.parse::<u64>().expect("Can’t parse twitter source ID"))
            .collect();

        (ids.pop(), ids.pop())
        // penultimate, latest
    }

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

    pub fn sync(&self, conn: &PgConnection) {
        let (penultimate, latest) = Self::latest_2_ids_in_db(conn);
        let latest = latest.or(penultimate).unwrap_or(0);
        println!("Latest twitter ID we have:\t\t{}", latest);
        if penultimate.is_some() {
            println!(
                "Penultimate twitter ID we have:\t\t{}",
                penultimate.unwrap()
            );
        }

        let mut statusbag: Vec<NewStatus> = vec![];
        let mut timeline = user_timeline(self.id, true, true, &self.token).with_page_size(200);
        let mut batch = 0;

        loop {
            // Get tweets older than penultimate, which should include the *latest*
            let (tl, feed) =
                block_on_all(timeline.older(penultimate)).expect("can’t read twitter timeline");
            batch += 1;
            timeline = tl;
            let mut contains_latest = false;
            let mut ntweets = 0;
            for tweet in &feed {
                ntweets += 1;
                statusbag.push((*tweet).into());
                if tweet.id == latest {
                    contains_latest = true;
                }
            }

            println!(
                "Batch {} ({} tweets) contains latest? {}",
                batch, ntweets, contains_latest
            );

            if contains_latest || ntweets == 0 || statusbag.len() >= 3200 {
                break;
            }
        }

        println!(
            "Made {} calls to twitter and retrieved {} tweets",
            batch,
            statusbag.len()
        );

        use diesel::insert_into;
        use crate::schema::statuses::dsl::*;

        let inserted = insert_into(statuses)
            .values(&statusbag)
            .on_conflict(source_id)
            .do_nothing()
            .execute(conn)
            .expect("Failed to insert tweets in db");

        let hint = if inserted == statusbag.len() - 1 {
            "as expected"
        } else if inserted >= statusbag.len() {
            "something’s odd"
        } else {
            "some duplicates"
        };

        println!("Inserted {} new tweets in DB ({})", inserted, hint);
    }
}
