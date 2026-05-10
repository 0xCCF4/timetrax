use crate::cli::ExecutableCommand;
use crate::cli::json_output::{activity_json, print_json};
use crate::cli::time_input::{TimeAt, TimeOffset, resolve_time};
use crate::data::app_config::AppConfig;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use clap::Parser;
use log::{error, info};
use serde_json::json;
use time::OffsetDateTime;

#[derive(Parser)]
pub struct CommandPop {
    /// Stop the activity at a specific time (HH:MM or HH:MM:SS)
    #[arg(long, conflicts_with = "offset")]
    at: Option<TimeAt>,
    /// Stop the activity offset from now (e.g. -5m, +1h30m, -90s)
    #[arg(long)]
    offset: Option<TimeOffset>,
    /// Delete the activity instead of stopping it when the end time is before the start time
    #[arg(long)]
    force: bool,
}

impl ExecutableCommand for CommandPop {
    type Error = std::io::Error;
    type Output = ();
    fn execute(
        &self,
        config: &AppConfig,
        job_config: &mut JobConfig,
        mut manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        let today_date = OffsetDateTime::now_local()
            .unwrap_or_else(|e| {
                error!("Failed to get local time. Falling back to UTC: {e}");
                OffsetDateTime::now_utc()
            })
            .date();

        let today = manager.get_or_create_day(today_date);

        if !today.inner().activities.is_empty() {
            let today = today.inner_mut();
            today
                .activities
                .sort_by_key(|a| a.time.start);

            let last_open_idx = today
                .activities
                .iter()
                .enumerate().rfind(|(_, a)| !a.time.is_complete())
                .map(|(i, _)| i);

            if let Some(idx) = last_open_idx {
                let end = resolve_time(self.at, self.offset);

                if end < today.activities[idx].time.start {
                    if self.force {
                        let removed = today.activities.remove(idx);
                        info!("Deleted activity (end before start): {removed:?}");
                        if config.json {
                            print_json(&json!({ "deleted": true, "activity": activity_json(&removed, today_date, job_config, None) }));
                        } else {
                            println!("Deleted activity: {removed}");
                        }
                    } else {
                        let start = today.activities[idx].time.start;
                        eprintln!(
                            "Error: end time {end} is before activity start {start}. \
                             Use --force to delete the activity instead."
                        );
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::InvalidInput,
                            "end time is before activity start",
                        ));
                    }
                } else {
                    info!("Popping activity: {:?}", today.activities[idx]);
                    today.activities[idx].time.complete_at(end);
                    if config.json {
                        print_json(&activity_json(&today.activities[idx], today_date, job_config, None));
                    } else {
                        println!("Stopped activity: {}", today.activities[idx]);
                        if today.activities.iter().all(|a| a.time.is_complete()) {
                            println!("All activities for today are complete.");
                        }
                    }
                }
            }
        }

        Ok(())
    }
}
