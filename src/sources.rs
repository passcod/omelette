use diesel::prelude::*;
use egg_mode::{tweet::user_timeline, user::UserID, KeyPair, Token};
use inserts::NewStatus;
use std::env;
use tokio::runtime::current_thread::block_on_all;

pub struct Twitter {
    token: Token,
    id: UserID<'static>,
}

impl Twitter {
    pub fn load() -> Self {
        let con_token = KeyPair::new(
            env::var("TWITTER_CONSUMER_KEY").expect("TWITTER_CONSUMER_KEY must be set"),
            env::var("TWITTER_CONSUMER_SECRET").expect("TWITTER_CONSUMER_SECRET must be set"),
        );
        let access_token = KeyPair::new(
            env::var("TWITTER_ACCESS_TOKEN_KEY").expect("TWITTER_ACCESS_TOKEN_KEY must be set"),
            env::var("TWITTER_ACCESS_TOKEN_SECRET").expect("TWITTER_ACCESS_TOKEN_SECRET must be set"),
        );
        let token = Token::Access {
            consumer: con_token,
            access: access_token,
        };

        let uid: u64 = env::var("TWITTER_USER_ID")
            .expect("TWITTER_USER_ID must be set")
            .parse()
            .expect("TWITTER_USER_ID must be u64");

        Self { token, id: uid.into() }
    }

    fn latest_id_in_db(conn: &PgConnection) -> Option<u64> {
        use models::{pg_repeat, pg_to_number};
        use schema::statuses::dsl::*;
        use types::Source;

        statuses.select(source_id)
            .filter(source.eq(Source::Twitter))
            .filter(is_repost.eq(false))
            // Awful, but less awful than implementing the cast function:
            .order_by(pg_to_number(source_id, pg_repeat("9", 25)).desc())
            .limit(1)
            .load::<String>(conn)
            .expect("Can’t retrieve last twitter source ID from db")
            .get(0)
            .map(|sid| sid.parse::<u64>().expect("Can’t parse twitter source ID"))
    }

    pub fn sync(&self, conn: &PgConnection) {
        let latest = Self::latest_id_in_db(conn);
        if latest.is_some() { println!("Latest twitter ID we have is {}", latest.unwrap()); }

        let timeline = user_timeline(self.id, true, true, &self.token)
            .with_page_size(200)
            .older(latest); // Confusingly, this says "get tweets newer than latest"

        let (_timeline, feed) = block_on_all(timeline).expect("can’t read twitter timeline");

        let mut statii: Vec<NewStatus> = Vec::with_capacity(200);
        for tweet in &feed {
            statii.push((*tweet).into());
            println!("<@{}> {}", tweet.user.as_ref().unwrap().screen_name, tweet.text);
        }

        use diesel::insert_into;
        use schema::statuses::dsl::*;

        let inserted = insert_into(statuses)
            .values(&statii)
            .on_conflict(source_id)
            .do_nothing()
            .execute(conn)
            .expect("Failed to insert tweets in db");

        println!("Inserted {} new tweets in DB", inserted);
    }
}
