use crate::command::{CommandPop, CommandPush, CommandStatus, ExecutableCommand};
use clap::Parser;
use log::{debug, error, info, trace};
use std::fs;
use std::path::PathBuf;
use timetrax::data::app_config::AppConfig;
use timetrax::data::job_config::JobConfig;
use timetrax::data::manager::Manager;

pub mod command;

#[derive(Parser)]
#[command(name = "TimeTrax")]
pub struct Args {
    #[command(subcommand)]
    pub command: Command,

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
    Push(CommandPush),
    /// Pop the most recent activity from the stack
    Pop(CommandPop),
    /// Status of current activities
    Status(CommandStatus),
}

impl ExecutableCommand for Command {
    type Error = std::io::Error;
    type Output = ();
    fn execute(&self, config: &AppConfig, manager: Manager) -> Result<Self::Output, Self::Error> {
        match self {
            Command::Push(cmd) => cmd.execute(config, manager),
            Command::Pop(cmd) => cmd.execute(config, manager),
            Command::Status(cmd) => cmd.execute(config, manager),
        }
    }
}

fn main() {
    env_logger::init();

    debug!(
        "Starting TimeTrax application ({})",
        option_env!("CARGO_PKG_VERSION").unwrap_or("<UNKNOWN>")
    );

    let args = Args::parse();
    let config = AppConfig::default();

    let data_path = args.data_path.unwrap_or_else(|| {
        trace!("No data path provided, using default.");
        config.default_data_path.clone()
    });

    debug!("Using data path: {:?}", data_path);

    let data_dir_exists = match fs::exists(&data_path) {
        Ok(exists) => exists,
        Err(err) => {
            error!(
                "Failed to check if data path exists at {:?}: {}",
                data_path, err
            );
            std::process::exit(1);
        }
    };

    if !data_dir_exists {
        info!("Data path does not exist, creating directory.");
        if let Err(err) = fs::create_dir_all(&data_path) {
            error!(
                "Failed to create data directory at {:?}: {}",
                data_path, err
            );
            std::process::exit(1);
        }
    }

    let job_config_path = data_path.join(&config.job_config_file_name);
    if !job_config_path.exists() {
        info!(
            "Job config file does not exist at {:?}, creating default config.",
            job_config_path
        );

        trace!("Opening job config file {:?}", job_config_path);
        let job_config_file = match fs::File::create(&job_config_path) {
            Ok(file) => file,
            Err(err) => {
                error!("Failed to job config at {:?}: {}", job_config_path, err);
                std::process::exit(1);
            }
        };

        trace!("Writing job config to {:?}", job_config_path);
        if let Err(err) = serde_json::to_writer_pretty(job_config_file, &JobConfig::default()) {
            error!(
                "Failed to write default job config to {:?}: {}",
                job_config_path, err
            );
            std::process::exit(1);
        }
    }

    let manager = match Manager::open(&config, data_path) {
        Ok(mgr) => mgr,
        Err(err) => {
            error!("Failed to load data directory: {}", err);
            std::process::exit(1);
        }
    };

    args.command.execute(&config, manager);
}
