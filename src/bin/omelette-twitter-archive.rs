use diesel::prelude::*;
use omelette::sources::twitter::Twitter;
use std::{io::Read, path::PathBuf};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Read from .env in working directory
    #[structopt(long = "dotenv")]
    dotenv: bool,

    /// Only do “slim” pass (parsing from CSV)
    #[structopt(long = "only-slim")]
    only_slim: bool,

    /// Only do “hydrate” pass (hydrating slim entries by Twitter lookup)
    #[structopt(long = "only-hydrate")]
    only_hydrate: bool,

    /// Archive file. Either a CSV or a ZIP (containing a tweets.csv)
    #[structopt(name = "FILE", parse(from_os_str))]
    file: Option<PathBuf>,
}

fn main() {
    use dotenv::dotenv;
    use std::{ffi::OsStr, fs::File, process::exit};
    use tree_magic::match_filepath;
    use zip::read::ZipArchive;

    let opt = Opt::from_args();

    if cfg!(debug_assertions) || opt.dotenv {
        println!("Loading .env");
        dotenv().ok();
    }

    if opt.only_slim && opt.only_hydrate {
        println!("!! Cannot use both --only-slim and --only-hydrate");
        exit(1);
    }

    let do_slim = !opt.only_hydrate;
    let do_hydrate = !opt.only_slim;

    let db = omelette::connect();
    let mut ids = Vec::with_capacity(0);

    if do_slim {
        let path = opt.file.expect("!! Missing path to archive file");
        let ext_csv = path.extension() == Some(OsStr::new("csv"));
        let is_csv = match_filepath("text/csv", &path);
        let is_zip = match_filepath("application/zip", &path);

        if !is_zip && !is_csv && !ext_csv {
            println!("!! File is neither a zip nor a csv, abort.");
            exit(1);
        }

        let file = File::open(&path).expect("!! File does not exist");

        ids = if is_zip {
            let mut archive = ZipArchive::new(file).unwrap();
            let entry = archive.by_name("tweets.csv").unwrap_or_else(|_| {
                println!("!!File is not a twitter archive, abort.");
                exit(1);
            });

            slim_load(&db, csv::Reader::from_reader(entry))
        } else {
            slim_load(&db, csv::Reader::from_reader(file))
        };
    }

    if do_hydrate {
        let tw = Twitter::load_unboxed().expect("!! Cannot connect to Twitter");
        // retrieve more info for each ("full" pass)

        // first using the ids from above
        if !ids.is_empty() {
            println!("\n=> Hydrating {} newly archived tweets (~{})", ids.len(), hydrate_est(ids.len()));
            for (i, batch) in ids.chunks(100).enumerate() {
                println!("-> Batch {} of {} tweets", i, batch.len());
                hydrate_batch(&db, &tw, batch);
            }
        }

        // then going back to the db and querying for slim-pass ones that may
        // have been missed or when the hydrate-pass was disabled.
        use omelette::schema::statuses::dsl::*;
        use omelette::types::{IntermediarySource, Source};
        let ids_left = statuses.select(id)
            .filter(source.eq(Source::Twitter))
            .filter(fetched_via.eq(IntermediarySource::TwitterArchive))
            .filter(source_author.eq(slim_mark()))
            .load::<i32>(&db)
            .expect("!! Cannot query DB for leftover slim entries");

        if !ids_left.is_empty() {
            println!("\n=> Hydrating {} leftover slim tweets (~{})", ids_left.len(), hydrate_est(ids_left.len()));
            for (i, batch) in ids_left.chunks(100).enumerate() {
                println!("-> Batch {} of {} tweets", i, batch.len());
                hydrate_batch(&db, &tw, batch);
            }
        }

        println!("\n=> Done hydrating.")
    }
}

fn slim_mark() -> String {
    "~slim~".into()
}

fn slim_load<R: Read>(conn: &PgConnection, csv_reader: csv::Reader<R>) -> Vec<i32> {
    use chrono::{TimeZone, Utc};
    use omelette::inserts::NewStatus;
    use omelette::schema::statuses::dsl::*;
    use omelette::types::{IntermediarySource, Source};
    use regex::Regex;

    let source_app_re = Regex::new("^<a href=\"([^\"]+)\".*>(.+)</a>$").unwrap();

    let mut ids = Vec::new();
    let mut bag = Vec::with_capacity(1000);
    let mut batch = 0;

    println!("\n=> Loading from archive in batches of 1000");
    let mut csv_reader = csv_reader;
    for record in csv_reader.records() {
        let record = record.expect("!! Error parsing CSV");

        let tweet_id = record[0].into();
        let in_reply_to_status_id = record[1].into();

        // Without the reply user name, can’t really write out the usual format.
        // So we'll get to that field (2) during another pass.

        let timestamp = Utc
            .datetime_from_str(&record[3], "%Y-%m-%d %H:%M:%S %z")
            .expect("!! Cannot parse date");
        let app = source_app_re
            .captures(&record[4])
            .map(|cap| format!("{} <{}>", &cap[2], &cap[1]))
            .unwrap_or("".into());
        let content = record[5].into();

        // The archive format confuses retweets and quoted tweets, so we can't
        // trust the retweet-related fields in there (6-8), and we'll get the
        // entities (field 9) in a latter pass.

        let status = NewStatus {
            text: content,
            author_id: None,
            geolocation_lat: None,
            geolocation_lon: None,
            posted_at: timestamp,
            fetched_at: Utc::now(),
            fetched_via: Some(IntermediarySource::TwitterArchive),
            deleted_at: None,
            is_repost: false,
            reposted_at: None,
            is_marked: false,
            marked_at: None,
            source: Source::Twitter,
            source_id: tweet_id,
            source_author: slim_mark(),
            source_app: app,
            in_reply_to_status: Some(in_reply_to_status_id),
            in_reply_to_user: None,
            quoting_status: None,
            public: true,
        };

        bag.push(status);
        if bag.len() >= 1000 {
            batch += 1;
            print!("-> Saving batch {}... ", batch);

            let mut results: Vec<i32> = diesel::insert_into(statuses)
                .values(&bag)
                .on_conflict(source_id)
                .do_nothing()
                .returning(id)
                .get_results(conn)
                .expect("\n!! Cannot save to database");

            ids.append(&mut results);
            bag.truncate(0);
            println!("done. {} new tweets loaded so far", ids.len());
        }
    }

    // store whatever remains
    if !bag.is_empty() {
        print!("-> Saving last batch ({} entries)... ", bag.len());
        let mut results: Vec<i32> = diesel::insert_into(statuses)
            .values(&bag)
            .on_conflict(source_id)
            .do_nothing()
            .returning(id)
            .get_results(conn)
            .expect("!! Cannot save to database");

        ids.append(&mut results);
        println!("done.");
    }

    println!(
        "=> {} entries processed and {} new tweets stored.",
        batch * 1000 + bag.len(),
        ids.len()
    );
    ids
}

fn hydrate_est(n: usize) -> String {
    let mut seconds = n * 5 / 100;
    let hours = seconds / 3600;
    seconds = seconds % 3600;
    let minutes = seconds / 60 + if seconds % 60 > 30 { 1 } else { 0 };

    if hours == 0 {
        format!("{} minutes", minutes)
    } else {
        format!("{} hours {} minutes", hours, minutes)
    }
}

fn hydrate_batch(conn: &PgConnection, tw: &Twitter, ids: &[i32]) {
    use std::time::{Duration, Instant};
    use std::thread::sleep;

    let min = Duration::from_secs(5);
    let now = Instant::now();

    // ...

    if let Some(left) = min.checked_sub(now.elapsed()) {
        sleep(left);
    }
}