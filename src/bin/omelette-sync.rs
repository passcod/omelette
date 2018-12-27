extern crate dotenv;
extern crate omelette;
extern crate structopt;

use dotenv::dotenv;
use omelette::sources::all_available;
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
    let sources = all_available();

    for (name, source) in &sources {
        println!("Syncing {:?}", name);
        source.sync(&db);
    }
}
