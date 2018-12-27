use chrono::{Duration, Utc};
use diesel::prelude::*;
use dotenv::dotenv;
use egg_mode_text::{entities, EntityKind};
use omelette::inserts::NewDeletion;
use omelette::models::{Entity, Status};
use regex::Regex;
use std::env;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Read from .env in working directory
    #[structopt(long = "dotenv")]
    dotenv: bool,
}

fn main() {
    let opt = Opt::from_args();

    if cfg!(debug_assertions) || opt.dotenv {
        println!("Loading .env");
        dotenv().ok();
    }

    let db = omelette::connect();

    let twitter_uid: u64 = env::var("TWITTER_USER_ID")
        .expect("TWITTER_USER_ID must be set")
        .parse()
        .expect("TWITTER_USER_ID must be u64");

    let requests: Vec<(Status, Option<Entity>)> = {
        use omelette::schema::entities;
        use omelette::schema::statuses::dsl::*;

        statuses
            .left_join(entities::table)
            .filter(deleted_at.is_null())
            .filter(entities::blob_hash.is_not_null().or(entities::id.is_null()))
            .filter(source_author.like(&format!("% ({})", twitter_uid)))
            .filter(text.like("%#cleanup%"))
            .load(&db)
            .expect("Cannot search statuses")
    };

    if requests.is_empty() {
        println!("No matching statuses, skip.");
        return;
    }

    let re = Regex::new(r"#cleanup(?:\s+(\d+)(s|m|h|d))?").unwrap();

    let now = Utc::now();
    let matches: Vec<Vec<NewDeletion>> = requests
        .into_iter()
        .filter_map(|(status, _)| {
            if entities(&status.text).into_iter().any(|ent| {
                ent.kind == EntityKind::Hashtag && ent.substr(&status.text) == "#cleanup"
            }) {
                let delay = match re.captures_iter(&status.text).next() {
                    None => {
                        println!("No duration, default to 15m\n“{}”", status.text);
                        900
                    }
                    Some(time) => {
                        let multiplier = match time.get(1).map(|s| s.as_str()).unwrap_or("m") {
                            "d" => 86400,
                            "h" => 3600,
                            "m" => 60,
                            "s" | _ => 1,
                        };

                        match time
                            .get(1)
                            .map(|s| s.as_str())
                            .unwrap_or("15")
                            .parse::<u32>()
                        {
                            Ok(n) => n * multiplier,
                            Err(err) => {
                                println!(
                                    "Cannot parse duration, default to 15m: {:?}\n“{}”",
                                    err, status.text
                                );
                                900
                            }
                        }
                    }
                };

                let not_before = now + Duration::seconds(delay as i64);
                let mut thread = vec![NewDeletion::from_status(&status, not_before)];
                println!(
                    "Requesting deletion: {:?} {} (#{})\n“{}” — {}",
                    status.source, status.source_id, status.id, status.text, status.posted_at
                );

                let mut stat = status.clone();
                loop {
                    match own_parent(&db, &twitter_uid, &stat) {
                        Threading::Stop => break,
                        Threading::Abort => return None,
                        Threading::Parent(s) => {
                            stat = s;
                            thread.push(NewDeletion::from_status(&stat, not_before));
                            println!(
                                "Requesting deletion: {:?} {} (#{})\n“{}” — {}",
                                stat.source, stat.source_id, stat.id, stat.text, stat.posted_at
                            );
                        }
                    }
                }

                Some(thread)
            } else {
                None
            }
        })
        .collect();

    if matches.is_empty() {
        println!("No matching statuses, skip.");
        return;
    }

    let matching = matches.len();
    let deletes: Vec<NewDeletion> = matches.into_iter().flatten().collect();
    println!(
        "Found {} matching statuses, requesting deletion for {} statuses",
        matching,
        deletes.len()
    );

    use omelette::schema::deletions::dsl::deletions;
    diesel::insert_into(deletions)
        .values(&deletes)
        .execute(&db)
        .expect("Cannot submit deletion requests");
}

enum Threading {
    Stop,
    Abort,
    Parent(Status),
}

fn own_parent(db: &PgConnection, twitter_uid: &u64, status: &Status) -> Threading {
    use omelette::schema::entities;
    use omelette::schema::statuses::dsl::*;

    if status.in_reply_to_status.is_none() {
        return Threading::Stop;
    }

    let parent_id = status.in_reply_to_status.clone().unwrap();
    let requests: Vec<(Status, Option<Entity>)> = statuses
        .left_join(entities::table)
        .filter(deleted_at.is_null())
        .filter(source.eq(&status.source))
        .filter(source_author.like(&format!("% ({})", twitter_uid)))
        .filter(source_id.eq(parent_id))
        .limit(1)
        .load(db)
        .expect("Cannot search statuses");

    if requests.is_empty() {
        return Threading::Stop;
    }

    let (stat, ent) = requests.first().unwrap();

    if let Some(entity) = ent {
        if entity.blob_hash.is_none() {
            println!(
                "Status has thin entities, skipping thread: {:?} {} (#{})\n“{}” — {}",
                stat.source, stat.source_id, stat.id, stat.text, stat.posted_at
            );

            return Threading::Abort;
        }
    }

    Threading::Parent(stat.clone())
}
