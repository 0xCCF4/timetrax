use clap::Parser;
use itertools::Itertools;
use std::path::PathBuf;
use timetrax::az_hash::AZHash;
use uuid::Uuid;

#[derive(Parser)]
#[command(name = "TimeTrax")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,
    /// Path to the folder to which time tracking data will be saved
    #[arg(short, long)]
    pub data_path: Option<PathBuf>,
}

#[derive(Parser)]
pub enum Command {
    /// Start the time tracking session
    Start,
    /// Stop the current time tracking session
    Stop,
    /// Show the current status of time tracking
    Status,
}

fn main() {
    let args = Args::parse();
    println!("Hello, world!");

    let uuid = (0..10).map(|_| Uuid::new_v4()).collect_vec();
    for uuid in uuid {
        println!("{} {}", uuid, uuid.az_hash())
    }
}
