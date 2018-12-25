extern crate omelette;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate dotenv;

use dotenv::dotenv;
use omelette::sources::Twitter;

fn main() {
    dotenv().ok();

    let db = omelette::connect();
    let twitter = Twitter::load();
    twitter.sync(&db);
}
