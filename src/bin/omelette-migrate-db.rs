#[macro_use]
extern crate diesel_migrations;

use dotenv::dotenv;
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Read from .env in working directory
    #[structopt(long = "dotenv")]
    dotenv: bool,
}

embed_migrations!();

fn main() {
    let opt = Opt::from_args();

    if cfg!(debug_assertions) || opt.dotenv {
        println!("Loading .env");
        dotenv().ok();
    }

    let db = omelette::connect();
    embedded_migrations::run_with_output(&db, &mut std::io::stdout()).unwrap();
    println!("=> Database is ready");
}
