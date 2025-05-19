// Disable before publishing; the dead code errors are noise while it's a WIP.
#![allow(dead_code)]

mod context;
mod db;
mod error;
mod file_entry;
mod ingest;
mod status;
mod timestamp;
mod snapshot;

use self::db::sql;
pub use self::error::{Error, Result};
use self::ingest::generate_checksums;

use crate::context::Context;
use crate::db::DB;

use anyhow::bail;
use clap::{Parser, Subcommand};

#[derive(Subcommand, Debug)]
pub enum Commands {
    /// Create database
    Initialize,

    /// Delete database
    Reset,

    /// Capture current file tree
    Index,

    /// Generate missing checksums. (May take awhile. Resumable)
    Checksum,

    /// Report current state of the database.
    Status,

    /// Used for development; if you're seeing this it means I forgot to delete it.
    Scratch,
}

#[derive(Parser, Debug)]
#[command(version, about)]
pub struct Args {
    #[command(subcommand)]
    pub command: Commands,

    #[arg(short = 'r', long = "repo", global = true)]
    /// Which repo in config.yml we are working with. Required if there is more than one.
    pub repo_name: Option<String>,

    #[arg(short = 'v', long = "verbose", global = true)]
    pub verbose: bool,
}

fn main() -> Result<()> {
    let ctx = Context::new(Args::parse()).expect("Unable to build context.");
    let mut db = DB::new(&ctx.db_path).unwrap();

    match ctx.args.command {
        Commands::Initialize => {
            if db.is_initialized()? {
                bail!("Database is already initialized.")
            }

            db.initialize()
        }
        Commands::Reset => DB::reset(db),
        Commands::Index => {
            if !db.is_initialized()? {
                bail!("Database must be initialized before indexing.")
            }

            ingest::index(&ctx, &mut db)
        }
        Commands::Checksum => {
            if !db.is_initialized()? {
                bail!("Database must be initialized before checksumming.")
            }

            generate_checksums(&ctx, &mut db)
        }
        Commands::Status => status::report(&ctx, &db),
        Commands::Scratch => Ok(()),
    }
}
