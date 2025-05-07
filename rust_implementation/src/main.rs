use self::context::Context;
use self::db::DB;

pub mod context;
pub mod db;
pub mod ingest;

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
    let ctx = Context::new(args.repo_name);
    let db = DB::new(&ctx.db_path);

    match args.command {
        Commands::Initialize => {
            let _ = db.initialize();
        }
        Commands::Reset => {
            let _ = DB::reset(db);
        }
        Commands::Index => {
            ingest::index(&ctx, &db);
        }
        _ => unimplemented!("WIP"),
    }
}
