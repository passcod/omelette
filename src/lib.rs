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
