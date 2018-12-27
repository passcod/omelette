extern crate omelette;
#[macro_use]
extern crate clap;
#[macro_use]
extern crate log;
extern crate dotenv;

use dotenv::dotenv;

fn main() {
    dotenv().ok();

    let db = omelette::connect();

    let sources = omelette::sources::all_available();
    for (name, source) in &sources {
        println!("Syncing {:?}", name);
        source.sync(&db);
    }
}
