use dotenv::dotenv;
use omelette::sources::{all_available, run_deletes, ActionMode};
use structopt::StructOpt;

#[derive(StructOpt, Debug)]
#[structopt()]
struct Opt {
    /// Don’t actually perform deletes
    #[structopt(long = "dry-run")]
    dry_run: bool,

    /// Ask before performing each delete
    #[structopt(long = "interactive")]
    interactive: bool,

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

    if opt.interactive && opt.dry_run {
        println!("!! Cannot supply both --dry-run and --interactive");
        std::process::exit(1);
    }

    let db = omelette::connect();
    let sources = all_available();

    run_deletes(
        &sources,
        &db,
        if opt.dry_run {
            ActionMode::DryRun
        } else if opt.interactive {
            ActionMode::Interactive
        } else {
            ActionMode::Auto
        },
    );
}
