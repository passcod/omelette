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
    for (name, srcs) in &sources {
        for src in srcs {
            println!(
                "\n=> Syncing {:?} {}",
                name,
                src.intermediary()
                    .map(|i| format!("(intermediary: {:?})", i))
                    .unwrap_or("".into())
            );

            if src.sync(&db) {
                successes += 1;
            }
        }
    }

    println!("\n=> Synced {} sources and intermediaries.", successes);
}
