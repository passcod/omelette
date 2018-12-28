use chrono::{TimeZone, Utc};
use diesel::prelude::*;
use dotenv::dotenv;
use omelette::{
    sources::twitter, types::{IntermediarySource, Source},
};
use regex::Regex;
use std::{
    ffi::OsStr, fs::File, io::{self, Read}, path::{Path, PathBuf}, process::exit,
};
use structopt::StructOpt;
use tree_magic::match_filepath;
use zip::read::{ZipArchive, ZipFile};

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Read from .env in working directory
    #[structopt(long = "dotenv")]
    dotenv: bool,

    /// Archive file. Either a CSV or a ZIP (containing a tweets.csv)
    #[structopt(name = "FILE", parse(from_os_str))]
    file: PathBuf,
}

fn main() {
    let opt = Opt::from_args();

    if cfg!(debug_assertions) || opt.dotenv {
        println!("Loading .env");
        dotenv().ok();
    }

    let ext_csv = opt.file.extension() == Some(OsStr::new("csv"));
    let is_csv = match_filepath("text/csv", &opt.file);
    let is_zip = match_filepath("application/zip", &opt.file);

    if !is_zip && !is_csv && !ext_csv {
        println!("!! File is neither a zip nor a csv, abort.");
        exit(1);
    }

    let file = File::open(&opt.file).expect("!! File does not exist");
    let db = omelette::connect();

    let ids = if is_zip {
        let mut archive = ZipArchive::new(file).unwrap();
        let entry = archive.by_name("tweets.csv").unwrap_or_else(|_| {
            println!("!!File is not a twitter archive, abort.");
            exit(1);
        });

        slim_load(&db, csv::Reader::from_reader(entry))
    } else {
        slim_load(&db, csv::Reader::from_reader(file))
    };

    // let tw = twitter::Twitter::load_unboxed();
    // retrieve more info for each ("full" pass)
}

fn slim_load<R: Read>(conn: &PgConnection, csv_reader: csv::Reader<R>) -> Vec<i32> {
    use omelette::inserts::NewStatus;
    use omelette::schema::statuses::dsl::*;

    let source_app_re = Regex::new("^<a href=\"([^\"]+)\".*>(.+)</a>$").unwrap();

    let mut ids = Vec::new();
    let mut bag = Vec::with_capacity(1000);
    let mut batch = 0;

    println!("=> Loading from archive in batches of 1000");
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
            source_author: "".into(),
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
