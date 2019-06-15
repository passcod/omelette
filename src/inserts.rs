#![allow(proc_macro_derive_resolution_fallback)]

use chrono::prelude::*;
use crate::models::Status;
use crate::schema::*;
use crate::types::*;
use egg_mode::{
    entities::MediaEntity,
    tweet::{ExtendedTweetEntities, Tweet},
    user::TwitterUser as EggUser,
};

#[derive(AsChangeset, Clone, Debug, Insertable, PartialEq, PartialOrd)]
#[table_name = "statuses"]
#[changeset_options(treat_none_as_null="true")]
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

impl From<&Tweet> for NewStatus {
    fn from(tweet: &Tweet) -> NewStatus {
        let (lat, lon) = match tweet.coordinates {
            None => (None, None),
            Some((a, o)) => (Some(a), Some(o)),
        };

        let (is_repost, otweet) = match tweet.retweeted_status {
            None => (false, Box::new(tweet.clone())),
            Some(ref tw) => (true, tw.clone()),
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
            reposted_at: if is_repost {
                Some(tweet.created_at)
            } else {
                None
            },
            is_marked: if let Some(liked) = tweet.favorited {
                liked
            } else {
                false
            },
            marked_at: None,
            source: Source::Twitter,
            source_id: format!("{}", otweet.id),
            source_author: if let Some(ref user) = otweet.user {
                format!("\"{}\" <@{}> ({})", user.name, user.screen_name, user.id)
            } else {
                "".into()
            },
            source_app: format!("{} <{}>", otweet.source.name, otweet.source.url),
            in_reply_to_status: if let Some(id) = otweet.in_reply_to_status_id {
                Some(format!("{}", id))
            } else {
                None
            },
            in_reply_to_user: if let Some(ref name) = otweet.in_reply_to_screen_name {
                Some(format!(
                    "{} <@{}>",
                    name,
                    otweet.in_reply_to_user_id.unwrap()
                ))
            } else {
                None
            },
            quoting_status: if let Some(id) = otweet.quoted_status_id {
                Some(format!("{}", id))
            } else {
                None
            },
            public: if let Some(ref user) = otweet.user {
                !user.protected
            } else {
                println!(
                    "~~ Cannot know whether tweet {:?} is public, defaulting to false",
                    otweet
                );
                false
            },
        }
    }
}

#[derive(Clone, Debug, Insertable, PartialEq, PartialOrd)]
#[table_name = "entities"]
pub struct NewEntity {
    pub fetched_at: DateTime<Utc>,
    pub status_id: i32,
    pub ordering: Option<i32>,
    pub media_type: MediaType,
    pub source_id: String,
    pub source_url: String,
    pub original_status_source_id: Option<String>,
    pub original_status_source_url: Option<String>,
}

impl From<&MediaEntity> for NewEntity {
    fn from(ent: &MediaEntity) -> NewEntity {
        let media_type = (&ent.media_type).into();
        let source_url = match media_type {
            MediaType::Photo => ent.media_url_https.clone(),
            MediaType::Gif => ent
                .video_info
                .clone()
                .unwrap()
                .variants
                .iter()
                .find(|v| v.bitrate == Some(0))
                .expect("!! GIF entity not in expected format")
                .url
                .clone(),
            MediaType::Video => {
                let mut variants = ent.video_info.clone().unwrap().variants;
                variants.sort_unstable_by_key(|v| v.bitrate.unwrap_or(0));
                variants
                    .last()
                    .map(|v| v.url.clone())
                    .unwrap_or(ent.media_url_https.clone())
            }
        };

        NewEntity {
            fetched_at: Utc::now(),
            status_id: 0,
            ordering: None,
            media_type,
            source_id: format!("{}", ent.id),
            source_url,
            original_status_source_id: ent.source_status_id.map(|id| format!("{}", id)),
            original_status_source_url: Some(ent.url.clone()),
        }
    }
}

impl NewEntity {
    pub fn from_extended(ents: &ExtendedTweetEntities) -> Vec<NewEntity> {
        ents.media
            .iter()
            .enumerate()
            .map(|(i, ent)| {
                let mut new_ent: NewEntity = ent.into();
                new_ent.ordering = Some(i as i32);
                new_ent
            })
            .collect()
    }
}

#[derive(Clone, Debug, Insertable, PartialEq, PartialOrd)]
#[table_name = "deletions"]
pub struct NewDeletion {
    pub not_before: DateTime<Utc>,
    pub status_id: i32,
    pub sponsor: String,
}

impl NewDeletion {
    pub fn from_status(status: &Status, not_before: DateTime<Utc>) -> Self {
        Self {
            not_before,
            status_id: status.id,
            sponsor: "omelette".into(),
        }
    }
}

#[derive(AsChangeset, Clone, Debug, Insertable, PartialEq, PartialOrd)]
#[table_name = "twitter_users"]
pub struct NewTwitterUser {
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

impl From<&EggUser> for NewTwitterUser {
    fn from(u: &EggUser) -> NewTwitterUser {
        NewTwitterUser {
            source_id: format!("{}", u.id),
            screen_name: u.screen_name.clone(),
            name: u.name.clone(),
            description: u.description.clone(),
            location: u.location.clone(),
            url: u.url.clone(),
            is_verified: u.verified,
            is_protected: u.protected,
            is_coauthored: u.contributors_enabled,
            is_translator: u.is_translator,
            statuses_count: u.statuses_count,
            following_count: u.friends_count,
            followers_count: u.followers_count,
            likes_count: u.favourites_count,
            listed_count: u.listed_count,
            created_at: u.created_at,
            fetched_at: Utc::now(),
            blocked_at: None,
            muted_at: None,
            missing: false,
            ui_language: u.lang.clone(),
            ui_timezone: u.time_zone.clone(),
            withheld_in: u.withheld_in_countries.clone().map(|v| v.join(", ")),
            withheld_scope: u.withheld_scope.clone(),
        }
    }
}

#[derive(Clone, Debug, Insertable, PartialEq, PartialOrd)]
#[table_name = "twitter_users"]
pub struct NewTwitterUserID {
    pub source_id: String,
    pub screen_name: String,
    pub name: String,
    pub created_at: DateTime<Utc>,
    pub fetched_at: DateTime<Utc>,
}

impl From<u64> for NewTwitterUserID {
    fn from(uid: u64) -> NewTwitterUserID {
        NewTwitterUserID {
            source_id: format!("{}", uid),
            screen_name: crate::SLIM_MARK.into(),
            name: "".into(),
            created_at: Utc::now(),
            fetched_at: Utc::now(),
        }
    }
}
