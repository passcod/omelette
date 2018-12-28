use chrono::Utc;
use crate::inserts::{NewEntity, NewStatus};
use crate::models::Status;
use crate::sources::{twitter::Twitter, DeleteError, LoadError, StatusSource};
use crate::types::{IntermediarySource, Source};
use diesel::prelude::*;
use std::{collections::HashMap, env, path::PathBuf};

#[derive(Clone, Debug)]
pub struct TwitterArchive {
    twitter: Twitter,
    path: PathBuf,
}

impl TwitterArchive {
    pub fn source() -> Source {
        Source::Twitter
    }

    pub fn load() -> Result<Box<StatusSource>, LoadError> {
        let twitter = Twitter::load_unboxed()?;

        Ok(Box::new(Self {
            twitter,
            path: PathBuf::from(
                env::var("TWITTER_ARCHIVE").unwrap_or("./omelette/twitter-archive.zip".into()),
            ),
        }))
    }
}

impl StatusSource for TwitterArchive {
    fn intermediary(&self) -> Option<IntermediarySource> {
        Some(IntermediarySource::TwitterArchive)
    }

    fn sync(&self, conn: &PgConnection) -> bool {
        println!("~~ No archive file found at {}, skip.", self.path.display());
        false
    }

    fn delete(&self, _conn: &PgConnection, _status: &Status) -> Result<(), DeleteError> {
        unimplemented!()
    }
}
