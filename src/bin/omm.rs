extern crate omelette;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate dotenv;

use dotenv::dotenv;

fn main() {
    dotenv().ok();
    println!("Hello, world!");

    let db = omelette::connect();
}
