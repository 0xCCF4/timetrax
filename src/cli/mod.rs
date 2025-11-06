use crate::data::app_config::AppConfig;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use clap::Parser;
use std::path::PathBuf;

mod class;
mod completion;
mod pop;
mod project;
mod push;
mod status;

pub use class::*;
pub use completion::*;
pub use pop::*;
pub use project::*;
pub use push::*;
pub use status::*;

pub trait ExecutableCommand {
    type Error;
    type Output;
    fn execute(
        &self,
        config: &AppConfig,
        job_config: &mut JobConfig,
        manager: Manager,
    ) -> Result<Self::Output, Self::Error>;
}

#[derive(Parser)]
#[command(name = "TimeTrax", bin_name = "timetrax")]
pub struct AppArgs {
    #[command(subcommand)]
    pub command: Option<Command>,

    /// Path to the folder to which time tracking data will be saved
    #[arg(short, long)]
    pub data_path: Option<PathBuf>,
    // /// App configuration file. If not provided, default config will be used
    // #[arg(short, long)]
    // pub config: Option<PathBuf>,
}

#[derive(Parser)]
pub enum Command {
    /// Push new activity to the stack
    #[clap(aliases = ["pu"])]
    Push(CommandPush),
    /// Pop the most recent activity from the stack
    #[clap(aliases = ["po"])]
    Pop(CommandPop),
    /// Status of current activities
    #[clap(aliases = ["s", "st", "stat", "info", "i", "display"])]
    Status(CommandStatus),
    /// Manage projects
    #[command(subcommand, aliases = ["projects", "proj", "prj", "p"])]
    Project(CommandProject),
    /// Manage activity classes
    #[command(subcommand, aliases = ["classes", "cls", "c", "ac"])]
    Class(CommandClass),
    /// Generate shell competition scripts
    #[command(aliases = ["complete", "autocomplete", "shell", "completions"])]
    Completion(CommandCompletion),
}

impl Default for Command {
    fn default() -> Self {
        Command::Status(CommandStatus::default())
    }
}

impl ExecutableCommand for Command {
    type Error = std::io::Error;
    type Output = ();
    fn execute(
        &self,
        config: &AppConfig,
        job_config: &mut JobConfig,
        manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        match self {
            Command::Push(cmd) => cmd.execute(config, job_config, manager),
            Command::Pop(cmd) => cmd.execute(config, job_config, manager),
            Command::Status(cmd) => cmd.execute(config, job_config, manager),
            Command::Project(cmd) => cmd.execute(config, job_config, manager),
            Command::Class(cmd) => cmd.execute(config, job_config, manager),
            Command::Completion(cmd) => cmd.execute(config, job_config, manager),
        }
    }
}
