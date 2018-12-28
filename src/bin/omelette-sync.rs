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

    let mut successes = 0;
    for (name, source) in &sources {
        println!("\n=> Syncing {:?}", name);

        if source.sync(&db) {
            successes += 1;
        }
    }

    println!("\n=> Synced {} sources.", successes);
}
