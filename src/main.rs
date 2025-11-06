use clap::Parser;
use log::{debug, error, info, trace};
use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;
use timetrax::cli::{AppArgs, Command, ExecutableCommand};
use timetrax::data::app_config::AppConfig;
use timetrax::data::dirty::DirtyMarker;
use timetrax::data::job_config::JobConfig;
use timetrax::data::manager::Manager;

fn main() {
    env_logger::init();

    debug!(
        "Starting TimeTrax application ({})",
        option_env!("CARGO_PKG_VERSION").unwrap_or("<UNKNOWN>")
    );

    let args = AppArgs::parse();
    let config = AppConfig::default();

    if let Some(command) = &args.command {
        if let Command::Completion(_) = command {
            trace!("Completion command detected, skipping data path setup.");
            if let Err(err) = command.execute(
                &config,
                &mut JobConfig::default(),
                Manager {
                    app_config: &config,
                    days: BTreeMap::new(),
                    data_path: PathBuf::new(),
                },
            ) {
                error!("Command execution failed: {}", err);
                std::process::exit(1);
            }
            return;
        }
    }

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

    let job_config = match Manager::open_job_config(&config, &data_path) {
        Ok(job) => job,
        Err(err) => {
            error!("Failed to load job config: {}", err);
            std::process::exit(1);
        }
    };

    let mut job_config = DirtyMarker::from(job_config);

    let manager = match Manager::open(&config, &data_path) {
        Ok(mgr) => mgr,
        Err(err) => {
            error!("Failed to load data directory: {}", err);
            std::process::exit(1);
        }
    };

    let command = args.command.unwrap_or_else(|| {
        trace!("No command provided, defaulting.");
        Command::default()
    });

    if let Err(err) = command.execute(&config, &mut job_config, manager) {
        error!("Command execution failed: {}", err);
        std::process::exit(1);
    }

    if job_config.is_dirty() {
        trace!("Job config marked as dirty, saving changes.");

        let job_config_path = data_path.join(&config.job_config_file_name);
        let job_config_file = match fs::File::create(&job_config_path) {
            Ok(file) => file,
            Err(err) => {
                error!(
                    "Failed to open job config file at {:?} for writing: {}",
                    job_config_path, err
                );
                std::process::exit(1);
            }
        };

        if let Err(err) = serde_json::to_writer_pretty(job_config_file, &*job_config) {
            error!(
                "Failed to write updated job config to {:?}: {}",
                job_config_path, err
            );
            std::process::exit(1);
        }

        trace!(
            "Successfully saved updated job config to {:?}",
            job_config_path
        );
    }
}
