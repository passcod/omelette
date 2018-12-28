#[macro_use]
extern crate diesel;
#[macro_use]
extern crate diesel_derive_enum;

use diesel::pg::PgConnection;
use diesel::prelude::*;
use std::env;

pub mod inserts;
pub mod models;
pub mod schema;
pub mod sources;
pub mod store;
pub mod types;

pub fn connect() -> PgConnection {
    let database_url = env::var("DATABASE_URL").expect("!! DATABASE_URL must be set");
    PgConnection::establish(&database_url)
        .expect(&format!("!! Error connecting to {}", database_url))
}
