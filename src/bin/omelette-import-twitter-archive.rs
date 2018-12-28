use dotenv::dotenv;
use omelette::sources::twitter;
use std::{ffi::OsStr, path::PathBuf, process::exit};
use structopt::StructOpt;
use tree_magic::match_filepath;

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

    // if zip, open tweets.csv

    // stream parse csv

    let db = omelette::connect();

    // load entries ("slim" pass)

    let tw = twitter::Twitter::load();

    // retrieve more info for each ("full" pass)
}
