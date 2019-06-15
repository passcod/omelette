#![allow(proc_macro_derive_resolution_fallback)]

use chrono::prelude::*;
use crate::schema::*;
use crate::types::*;
use diesel::sql_types::*;

#[derive(Clone, Debug, Identifiable, Insertable, PartialEq, PartialOrd, Queryable)]
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

#[derive(Associations, Clone, Debug, Identifiable, Insertable, PartialEq, PartialOrd, Queryable)]
#[belongs_to(Status, foreign_key = "status_id")]
#[table_name = "entities"]
pub struct Entity {
    pub id: i32,
    pub fetched_at: DateTime<Utc>,
    pub status_id: i32,
    pub ordering: Option<i32>,
    pub media_type: MediaType,
    pub source_id: String,
    pub source_url: String,
    pub original_status_source_id: Option<String>,
    pub original_status_source_url: Option<String>,
    pub blob_hash: Option<String>,
}

#[derive(Associations, Clone, Debug, Identifiable, Insertable, PartialEq, PartialOrd, Queryable)]
#[belongs_to(Status, foreign_key = "status_id")]
#[table_name = "deletions"]
pub struct Deletion {
    pub id: i32,
    pub status_id: i32,
    pub created_at: DateTime<Utc>,
    pub not_before: DateTime<Utc>,
    pub executed_at: Option<DateTime<Utc>>,
    pub sponsor: String,
}

#[derive(Clone, Debug, Identifiable, Insertable, PartialEq, PartialOrd, Queryable)]
#[table_name = "twitter_users"]
pub struct TwitterUser {
    pub id: i32,
    pub source_id: String,
    pub screen_name: String,
    pub name: String,
    pub description: Option<String>,
    pub location: Option<String>,
    pub url: Option<String>,
    pub is_verified: bool,
    pub is_protected: bool,
    pub is_coauthored: bool,
    pub is_translator: bool,
    pub statuses_count: i32,
    pub following_count: i32,
    pub followers_count: i32,
    pub likes_count: i32,
    pub listed_count: i32,
    pub created_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
    pub blocked_at: Option<DateTime<Utc>>,
    pub muted_at: Option<DateTime<Utc>>,
    pub missing: bool,
    pub ui_language: Option<String>,
    pub ui_timezone: Option<String>,
    pub withheld_in: Option<String>,
    pub withheld_scope: Option<String>,
}

sql_function!(#[sql_name="repeat"] fn pg_repeat(t: Text, n: Int4) -> Text);
sql_function!(#[sql_name="to_number"] fn pg_to_number(t: Text, f: Text) -> Numeric);
