use crate::cli::{AppArgs, ExecutableCommand};
use crate::data::app_config::AppConfig;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use clap::{CommandFactory, Parser, ValueEnum};
use clap_complete::Shell;
use log::error;
use std::io::BufWriter;
use std::path::PathBuf;

#[derive(Parser)]
pub struct CommandCompletion {
    /// When set, generate shell completion for all supported shells
    #[arg(short, long, aliases = ["out", "output"])]
    output_dir: Option<PathBuf>,
    /// Generate completion for this specific shell and output it to stdout
    #[arg(short, long)]
    shell: Option<String>,
}

impl ExecutableCommand for CommandCompletion {
    type Error = std::io::Error;
    type Output = ();
    fn execute(
        &self,
        _config: &AppConfig,
        _job_config: &mut JobConfig,
        _manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        if let Some(output_dir) = &self.output_dir {
            for &shell in Shell::value_variants() {
                clap_complete::generate_to(shell, &mut AppArgs::command(), "timetrax", output_dir)?;
            }
        }

        if let Some(shell_name) = &self.shell {
            let shell = Shell::from_str(shell_name, true);
            let shell = match shell {
                Ok(shell) => shell,
                Err(_) => {
                    error!("Unsupported shell: {}", shell_name);

                    error!("Available shells are:");
                    for &s in Shell::value_variants() {
                        error!(" - {}", s);
                    }

                    return Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        format!("Unsupported shell: {}", shell_name),
                    ));
                }
            };

            let mut stdout = BufWriter::new(std::io::stdout());
            clap_complete::generate(shell, &mut AppArgs::command(), "timetrax", &mut stdout);
        }

        Ok(())
    }
}
