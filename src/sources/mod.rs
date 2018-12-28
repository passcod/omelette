use crate::models::{Deletion, Status};
use crate::types::Source;
use diesel::{pg::PgConnection, result::Error as DieselError};
use egg_mode::error::Error as EggError;
use std::{
    collections::HashMap, env::VarError, io::{self, Write},
};

pub mod twitter;

pub type Sources = HashMap<Source, Box<StatusSource>>;

pub fn all_available() -> Sources {
    let mut sources: Sources = HashMap::new();

    macro_rules! load_source {
        ($struct:ident) => {
            match $struct::load() {
                Err(err) => println!("!! Error loading {}: {:?}", stringify!($struct), err),
                Ok(source) => {
                    sources.insert($struct::source(), source);
                }
            };
        };
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
        .expect("!! Cannot load deletions from DB");

    if deletes.is_empty() {
        println!("!! No deletion requests ready, skip.");
        return;
    }

    println!("=> {} deletion requests ready for action", deletes.len());

    let mut successes = 0;
    for (delete, status) in &deletes {
        if let Some(source) = sources.get(&status.source) {
            match mode {
                ActionMode::DryRun => println!("\n-> DRY RUN: would delete status: {:?}", status),
                ActionMode::Interactive => {
                    println!(
                        "-> Request to delete {:?} status {} (internal id {}) from {}\n“{}” — {}",
                        status.source, status.source_id, status.id, delete.sponsor, status.text, status.posted_at
                    );

                    print!(
                        "
To delete now, type 'delete' or 'd'.
To dump the status, type 'show' or '?'.
To skip, type anything else or just press enter.
To exit, use Ctrl-C.

Delete status {}? ",
                        123
                    );

                    io::stdout().flush().unwrap();

                    let mut nline = String::with_capacity(7);
                    io::stdin()
                        .read_line(&mut nline)
                        .expect("!! Could not grab input");
                    let mut command = nline.trim();

                    if command == "show" || command == "?" {
                        print!("\n{:?}\n\nDelete status? ", status);
                        io::stdout().flush().unwrap();

                        nline.truncate(0);
                        io::stdin()
                            .read_line(&mut nline)
                            .expect("!! Could not grab input");
                        command = nline.trim();
                    }

                    if command == "delete" || command == "d" {
                        successes += one_delete(source, conn, status, delete);
                    }

                    println!("");
                }
                ActionMode::Auto => {
                    successes += one_delete(source, conn, status, delete);
                }
            };
        } else {
            println!("!! No source available for deletion #{}", delete.id);
        }
    }

    println!("\n=> {} successful deletes performed", successes);
}

fn one_delete(
    source: &Box<StatusSource>,
    conn: &PgConnection,
    status: &Status,
    delete: &Deletion,
) -> usize {
    use chrono::Utc;
    use crate::schema::deletions::dsl::*;
    use diesel::prelude::*;

    println!(
        "-> Deleting {:?} status {} (internal id {}) on request from {}",
        status.source, status.source_id, status.id, delete.sponsor
    );

    let record = diesel::update(deletions.find(delete.id)).set(executed_at.eq(Utc::now()));

    if let Err(err) = source.delete(conn, &status) {
        println!("!! Could not delete status: {:?}", err);

        if let DeleteError::AlreadyDone = err {
            record.execute(conn).expect(&format!(
                "!! Failed to record deletion status for #{}",
                delete.id
            ));
        }

        0
    } else {
        record.execute(conn).expect(&format!(
            "!! Failed to record deletion success for #{}",
            delete.id
        ));

        1
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
    fn sync(&self, conn: &PgConnection) -> bool;
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
