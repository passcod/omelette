extern crate egg_mode;
#[macro_use]
extern crate log;
extern crate egg_mode_text;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;
extern crate chrono;

use diesel::prelude::*;
use diesel::pg::PgConnection;
use egg_mode::{KeyPair, Token, user::UserID};
use std::env;

pub mod types;
pub mod schema;
pub mod models;
pub mod inserts;

pub fn connect() -> PgConnection {
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn twitter_token() -> Token {
    let con_token = KeyPair::new(
        env::var("TWITTER_CONSUMER_KEY")
            .expect("TWITTER_CONSUMER_KEY must be set"),
        env::var("TWITTER_CONSUMER_SECRET")
            .expect("TWITTER_CONSUMER_SECRET must be set")
    );
    let access_token = KeyPair::new(
        env::var("TWITTER_ACCESS_TOKEN_KEY")
            .expect("TWITTER_ACCESS_TOKEN_KEY must be set"),
        env::var("TWITTER_ACCESS_TOKEN_SECRET")
            .expect("TWITTER_ACCESS_TOKEN_SECRET must be set")
    );
    Token::Access {
        consumer: con_token,
        access: access_token,
    }
}

pub fn twitter_id() -> u64 {
    env::var("TWITTER_USER_ID")
        .expect("TWITTER_USER_ID must be set")
        .parse()
        .expect("TWITTER_USER_ID must be u64")
}
