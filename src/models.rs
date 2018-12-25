#![allow(proc_macro_derive_resolution_fallback)]

use diesel::sql_types::*;
use chrono::prelude::*;
use schema::statuses;
use types::{IntermediarySource, Source};

#[derive(Queryable, Insertable)]
#[table_name = "statuses"]
pub struct Status {
    pub id: i32,
    pub text: String,
    pub author_id: Option<i32>,
    pub geolocation_lat: Option<f64>,
    pub geolocation_lon: Option<f64>,
    pub posted_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
    pub fetched_via: Option<IntermediarySource>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub is_repost: bool,
    pub reposted_at: Option<DateTime<Utc>>,
    pub is_marked: bool,
    pub marked_at: Option<DateTime<Utc>>,
    pub source: Source,
    pub source_id: String,
    pub source_author: String,
    pub source_app: String,
    pub in_reply_to_status: Option<String>,
    pub in_reply_to_user: Option<String>,
    pub quoting_status: Option<String>,
    pub public: bool,
}

sql_function!(#[sql_name="repeat"] fn pg_repeat(t: Text, n: Int4) -> Text);
sql_function!(#[sql_name="to_number"] fn pg_to_number(t: Text, f: Text) -> Numeric);
