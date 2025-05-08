mod context;
mod db;
mod error;
mod ingest;

pub use self::error::{Error, Result};

use crate::context::Context;
use crate::db::DB;

use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
enum Commands {
    /// Create database
    Initialize,

    /// Delete database
    Reset,

    /// Capture current file tree
    Index,

    /// Generate missing checksums. (May take awhile. Resumable)
    Checksum,
}

#[derive(Parser, Debug)]
#[command(version, about)]
struct Args {
    #[command(subcommand)]
    command: Commands,
    #[arg(short = 'r', long = "repo")]
    /// Which repo in config.yml we are working with. Required if there is more than one.
    repo_name: Option<String>,
}

fn main() {
    let args = Args::parse();
    let ctx = Context::new(args.repo_name).expect("Unable to build context.");
    let mut db = DB::new(&ctx.db_path).unwrap();

    match args.command {
        Commands::Initialize => {
            let _ = db.initialize().unwrap();
        }
        Commands::Reset => {
            let _ = DB::reset(db);
        }
        Commands::Index => {
            let _ = ingest::index(&ctx, &mut db);
        }

        _ => unimplemented!("WIP"),
    }
}
