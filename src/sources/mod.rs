use crate::models::{Deletion, Status};
use crate::types::Source;
use diesel::{pg::PgConnection, result::Error as DieselError};
use egg_mode::error::Error as EggError;
use std::collections::HashMap;
use std::env::VarError;

pub mod twitter;

pub type Sources = HashMap<Source, Box<StatusSource>>;

pub fn all_available() -> Sources {
    let mut sources = HashMap::new();

    macro_rules! load_source {
        ($struct:ident) => {
            match $struct::load() {
                Err(err) => println!("Error loading {}: {:?}", stringify!($struct), err),
                Ok(source) => {sources.insert($struct::source(), source).unwrap(); },
            };
        }
    }

    use self::twitter::Twitter;
    load_source!(Twitter);

    sources
}

pub fn run_deletes(sources: &Sources, conn: &PgConnection, mode: ActionMode) {
    use chrono::Utc;
    use crate::schema::deletions::dsl::*;
    use crate::schema::statuses;
    use diesel::prelude::*;

    let deletes: Vec<(Deletion, Status)> = deletions
        .inner_join(statuses::table)
        .filter(executed_at.is_null())
        .filter(not_before.lt(Utc::now()))
        .order_by(not_before)
        .load(conn)
        .expect("Cannot load deletions from DB");

    if deletes.is_empty() {
        println!("No deletion requests ready, skip.");
        return;
    }

    println!("{} deletion requests ready for action", deletes.len());

    for (delete, status) in &deletes {
        if let Some(source) = sources.get(&status.source) {
            match mode {
                ActionMode::DryRun => println!("DRY RUN: would delete status: {:?}", status),
                ActionMode::Interactive => unimplemented!(),
                ActionMode::Auto => {
                    println!("Deleting {:?} status {} (internal id {})", status.source, status.source_id, status.id);
                    if let Err(err) = source.delete(conn, &status) {
                        println!("Could not delete status: {:?}", err);
                    }
                }
            };
        } else {
            println!("No source available for deletion #{}", delete.id);
        }
    }
}

#[derive(Clone, Debug)]
pub enum ActionMode {
    Auto,
    DryRun,
    Interactive,
}

impl Default for ActionMode {
    fn default() -> Self {
        ActionMode::Auto
    }
}

pub trait StatusSource {
    fn sync(&self, conn: &PgConnection);
    fn delete(&self, conn: &PgConnection, status: &Status) -> Result<(), DeleteError>;
}

#[derive(Debug)]
pub enum LoadError {
    Env(VarError),
}

impl From<VarError> for LoadError {
    fn from(err: VarError) -> LoadError {
        LoadError::Env(err)
    }
}

#[derive(Debug)]
pub enum DeleteError {
    AlreadyDone,
    WrongSource,
    Unimplemented,
    Database(DieselError),
    Twitter(EggError),
}

impl From<DieselError> for DeleteError {
    fn from(err: DieselError) -> DeleteError {
        DeleteError::Database(err)
    }
}

impl From<EggError> for DeleteError {
    fn from(err: EggError) -> DeleteError {
        DeleteError::Twitter(err)
    }
}
