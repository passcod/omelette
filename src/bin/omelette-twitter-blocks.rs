use dotenv::dotenv;
use omelette::sources::twitter::Twitter;
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
    let tw = Twitter::load_unboxed().expect("!! Cannot connect to Twitter");

    println!("\n=> Fetching blocks");
    println!("\n   This can be pretty slow as we do one call per ~minute to aggressively respect the rate-limiting.");
    let (fetched, inserted) = tw.fetch_blocks(&db);
    println!("\n=> Fetched {} new blocks, inserted {} new users.", fetched, inserted);
}
