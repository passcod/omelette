#![allow(proc_macro_derive_resolution_fallback)]

use chrono::prelude::*;
use egg_mode::tweet::Tweet;
use types::{Source, IntermediarySource};
use super::schema::statuses;

#[derive(Insertable)]
#[table_name="statuses"]
pub struct NewStatus {
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

impl From<Tweet> for NewStatus {
    fn from(tweet: Tweet) -> NewStatus {
        let (lat, lon) = match tweet.coordinates {
            None => (None, None),
            Some((a, o)) => (Some(a), Some(o))
        };

        let (is_repost, otweet) = match tweet.retweeted_status {
            None => (false, Box::new(tweet.clone())),
            Some(tw) => (true, tw)
        };

        NewStatus {
            text: otweet.text.clone(),
            author_id: None,
            geolocation_lat: lat,
            geolocation_lon: lon,
            posted_at: otweet.created_at,
            fetched_at: Utc::now(),
            fetched_via: None,
            deleted_at: None,
            is_repost,
            reposted_at: if is_repost { Some(tweet.created_at) } else { None },
            is_marked: if let Some(liked) = tweet.favorited { liked } else { false },
            marked_at: None,
            source: Source::Twitter,
            source_id: format!("{}", otweet.id),
            source_author: if let Some(ref user) = otweet.user {
                format!("\"{}\" <@{}> ({})", user.name, user.screen_name, user.id)
            } else { "".into() },
            source_app: format!("{} <{}>", otweet.source.name, otweet.source.url),
            in_reply_to_status: if let Some(id) = otweet.in_reply_to_status_id {
                Some(format!("{}", id))
            } else {
                None
            },
            in_reply_to_user: if let Some(ref name) = otweet.in_reply_to_screen_name {
                Some(format!("{} <@{}>", name, otweet.in_reply_to_user_id.unwrap()))
            } else {
                None
            },
            quoting_status: if let Some(id) = otweet.quoted_status_id {
                Some(format!("{}", id))
            } else {
                None
            },
            public: if let Some(ref user) = otweet.user {
                user.protected
            } else {
                warn!("Cannot know whether tweet {:?} is public, defaulting to false", otweet);
                false
            },
        }
    }
}
