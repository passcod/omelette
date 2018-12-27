use crate::models::Status;
use diesel::{pg::PgConnection, result::Error as DieselError};
use egg_mode::error::Error as EggError;
use std::env::VarError;

pub mod twitter;

pub fn all_available() -> Vec<Box<StatusSource>> {
    let mut sources = Vec::new();

    macro_rules! load_source {
        ($struct:ident) => {
            match $struct::load() {
                Err(err) => println!("Error loading {}: {:?}", stringify!($struct), err),
                Ok(source) => sources.push(source),
            };
        }
    }

    use self::twitter::Twitter;
    load_source!(Twitter);

    sources
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
