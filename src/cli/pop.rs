use crate::cli::ExecutableCommand;
use crate::data::app_config::AppConfig;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use clap::Parser;
use log::{error, info};
use time::OffsetDateTime;

#[derive(Parser)]
pub struct CommandPop {}

impl ExecutableCommand for CommandPop {
    type Error = std::io::Error;
    type Output = ();
    fn execute(
        &self,
        _config: &AppConfig,
        _job_config: &mut JobConfig,
        mut manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        let today = OffsetDateTime::now_local()
            .unwrap_or_else(|e| {
                error!("Failed to get local time. Falling back to UTC: {}", e);
                OffsetDateTime::now_utc()
            })
            .date();

        let today = manager.get_or_create_day(today);

        if !today.inner().activities.is_empty() {
            let today = today.inner_mut();
            today
                .activities
                .sort_by(|a, b| a.time.start.cmp(&b.time.start));

            if let Some(activity) = today
                .activities
                .iter_mut()
                .filter(|a| !a.time.is_complete())
                .last()
            {
                info!("Popping activity: {:?}", activity);
                activity.time.complete_now();

                println!("Stopped activity: {activity}");

                if today.activities.iter_mut().all(|a| a.time.is_complete()) {
                    println!("All activities for today are complete.");
                }
            }
        }

        Ok(())
    }
}
