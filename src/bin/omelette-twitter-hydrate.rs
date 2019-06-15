use diesel::prelude::*;
use omelette::sources::twitter::Twitter;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Read from .env in working directory
    #[structopt(long = "dotenv")]
    dotenv: bool,

    /// Hydrate users
    users: bool,

    /// Hydrate tweets
    tweets: bool,
}

fn main() {
    use dotenv::dotenv;

    let opt = Opt::from_args();

    if cfg!(debug_assertions) || opt.dotenv {
        println!("Loading .env");
        dotenv().ok();
    }

    let (do_users, do_tweets) = if !opt.users && !opt.tweets {
        println!("-- No hydration target provided, assuming all");
        (true, true)
    } else {
        (opt.users, opt.tweets)
    };

    let db = omelette::connect();
    let tw = Twitter::load_unboxed().expect("!! Cannot connect to Twitter");

    if do_tweets {
        use omelette::schema::statuses::dsl::*;
        use omelette::types::Source;
        let ids_left = statuses.select(id)
            .filter(source.eq(Source::Twitter))
            .filter(source_author.eq(omelette::slim()))
            .load::<i32>(&db)
            .expect("!! Cannot query DB for slim tweets");

        if !ids_left.is_empty() {
            println!("\n=> Hydrating {} slim tweets (~{})", ids_left.len(), hydrate_est(ids_left.len()));
            for (i, batch) in ids_left.chunks(100).enumerate() {
                println!("-> Batch {} of {} tweets", i + 1, batch.len());
                hydrate_batch_tweets(&db, &tw, batch);
            }
        }
    }

    if do_users {
        use omelette::schema::twitter_users::dsl::*;
        let ids_left = twitter_users.select(id)
            .filter(screen_name.eq(omelette::slim()))
            .load::<i32>(&db)
            .expect("!! Cannot query DB for slim users");

        if !ids_left.is_empty() {
            println!("\n=> Hydrating {} slim users (~{})", ids_left.len(), hydrate_est(ids_left.len()));
            for (i, batch) in ids_left.chunks(100).enumerate() {
                println!("-> Batch {} of {} users", i + 1, batch.len());
                hydrate_batch_users(&db, &tw, batch);
            }
        }
    }

    println!("\n=> Done hydrating.")
}

fn hydrate_est(n: usize) -> String {
    let mut seconds = n * 5 / 100;
    let hours = seconds / 3600;
    seconds = seconds % 3600;

    let mut minutes = seconds / 60;
    if seconds % 60 > 30 {
        minutes += 1;
    } else if minutes == 0 {
        minutes = 1;
    }

    if hours == 0 {
        format!("{} minutes", minutes)
    } else {
        format!("{} hours {} minutes", hours, minutes)
    }
}

fn hydrate_batch_tweets(conn: &PgConnection, tw: &Twitter, ids: &[i32]) {
    use chrono::Utc;
    use egg_mode::tweet::lookup_map;
    use omelette::inserts::{NewEntity, NewStatus};
    use omelette::models::Status;
    use omelette::types::IntermediarySource;
    use std::thread::sleep;
    use std::time::{Duration, Instant};
    use tokio::runtime::current_thread::block_on_all;

    // Rate-limit on lookups is 300 per app per 15 minutes. That works out at
    // about one run every 3 seconds. But the *user* limit is 900, and we don't
    // want to inconvenience the user, nor to forbid other omelette tools that
    // might use lookups during the same time, so we push up to 5 seconds.
    let min = Duration::from_secs(5);
    let now = Instant::now();

    let source_ids: Vec<u64> = {
        use omelette::schema::statuses::dsl::*;
        statuses.select(source_id)
            .filter(id.eq_any(ids))
            .load::<String>(conn)
            .expect("!! Cannot connect to DB")
            .iter()
            .map(|sid| sid.parse().expect("!! Cannot parse source ID"))
            .collect()
    };

    let tweets = match block_on_all(lookup_map(source_ids, &tw.token)) {
        Ok(tws) => tws,
        Err(err) => {
            println!("!! Cannot fetch tweets, skipping batch.\n{:?}", err);
            return;
        }
    };

    let statuses: Vec<Status> = {
        use omelette::schema::statuses::dsl::*;
        statuses
            .filter(id.eq_any(ids))
            .load(conn)
            .expect("!! Cannot connect to DB")
    };

    for status in &statuses {
        use omelette::schema::statuses::dsl::*;

        let sid: u64 = status.source_id.parse().expect("!! Cannot parse source ID");
        let tweet = tweets.get(&sid).expect("!! Mismatch between input and lookup");

        if let Some(tweet) = tweet {
            let mut insert: NewStatus = tweet.into();
            insert.fetched_via = Some(IntermediarySource::TwitterArchive);

            let mut entitybag = if let Some(ref ents) = tweet.extended_entities {
                NewEntity::from_extended(&ents)
            } else {
                Vec::new()
            };

            // Update and insert in a transaction so we don’t save an hydrated
            // tweet without its entities if we’re interrupted in the middle.
            conn.transaction::<_, diesel::result::Error, _>(|| {
                let new_id = if status.source_id == insert.source_id {
                    diesel::update(statuses.find(status.id))
                        .set(insert)
                        .execute(conn)?;

                    status.id
                } else {
                    // Hydrating to a retweet.
                    // We instead insert the hydrated tweet and delete the slim.

                    let nids: Vec<i32> = diesel::insert_into(statuses)
                        .values(&insert)
                        .on_conflict(source_id)
                        .do_nothing()
                        .returning(id)
                        .load(conn)?;

                    let nid = match nids.get(0) {
                        Some(n) => n.clone(),
                        None => statuses.select(id)
                            .filter(source_id.eq(insert.source_id))
                            .first::<i32>(conn)?
                    };

                    diesel::delete(statuses.find(status.id))
                        .execute(conn)?;

                    nid
                };

                // Add those late so it picks up if the ID has changed above
                for ent in &mut entitybag {
                    ent.status_id = new_id;
                }

                {
                    use omelette::schema::entities::dsl::*;
                    diesel::insert_into(entities)
                        .values(&entitybag)
                        .on_conflict(source_id)
                        .do_nothing()
                        .execute(conn)?;
                }

                Ok(())
            }).expect("!! Cannot update DB");
        } else {
            diesel::update(statuses.find(status.id))
                .set((
                    source_author.eq("".to_string()),
                    deleted_at.eq(Utc::now())
                ))
                .execute(conn)
                .expect("!! Cannot update DB");
        }
    }

    if let Some(left) = min.checked_sub(now.elapsed()) {
        sleep(left);
    }
}

fn hydrate_batch_users(conn: &PgConnection, tw: &Twitter, ids: &[i32]) {
    use chrono::Utc;
    use egg_mode::user::{lookup, TwitterUser as EggUser};
    use omelette::inserts::NewTwitterUser;
    use omelette::models::TwitterUser;
    use std::collections::HashMap;
    use std::thread::sleep;
    use std::time::{Duration, Instant};
    use tokio::runtime::current_thread::block_on_all;

    // Rate-limit on lookups is 300 per app per 15 minutes. That works out at
    // about one run every 3 seconds. But the *user* limit is 900, and we don't
    // want to inconvenience the user, nor to forbid other omelette tools that
    // might use lookups during the same time, so we push up to 5 seconds.
    let min = Duration::from_secs(5);
    let now = Instant::now();

    let source_ids: Vec<u64> = {
        use omelette::schema::twitter_users::dsl::*;
        twitter_users.select(source_id)
            .filter(id.eq_any(ids))
            .load::<String>(conn)
            .expect("!! Cannot connect to DB")
            .iter()
            .map(|sid| sid.parse().expect("!! Cannot parse source ID"))
            .collect()
    };

    let twuv = match block_on_all(lookup(source_ids, &tw.token)) {
        Ok(us) => us,
        Err(err) => {
            println!("!! Cannot fetch users, skipping batch.\n{:?}", err);
            return;
        }
    };

    let mut twusers = HashMap::with_capacity(twuv.len());
    for twu in twuv.into_iter() {
        twusers.insert(twu.id, twu);
    }
    let twusers = twusers;

    let dbusers: Vec<TwitterUser> = {
        use omelette::schema::twitter_users::dsl::*;
        twitter_users
            .filter(id.eq_any(ids))
            .load(conn)
            .expect("!! Cannot connect to DB")
    };

    for user in &dbusers {
        use omelette::schema::twitter_users::dsl::*;

        let sid: u64 = user.source_id.parse().expect("!! Cannot parse source ID");

        if let Some(twuser) = twusers.get(&sid) {
            let twu: &EggUser = &*twuser;
            let mut insert: NewTwitterUser = twu.into();

            insert.blocked_at = user.blocked_at;
            insert.muted_at = user.muted_at;

            diesel::update(twitter_users.find(user.id))
                .set(insert)
                .execute(conn)
                .expect("!! Cannot update DB");
        } else {
            diesel::update(twitter_users.find(user.id))
                .set((
                    missing.eq(true),
                    fetched_at.eq(Utc::now())
                ))
                .execute(conn)
                .expect("!! Cannot update DB");
        }
    }

    if let Some(left) = min.checked_sub(now.elapsed()) {
        sleep(left);
    }
}
