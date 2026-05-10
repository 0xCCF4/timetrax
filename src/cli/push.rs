use crate::cli::ExecutableCommand;
use crate::cli::json_output::{activity_json, print_json};
use crate::cli::time_input::{TimeAt, TimeOffset, resolve_time};
use crate::data::activity::Activity;
use crate::data::app_config::AppConfig;
use crate::data::identifier::Identifier;
use crate::data::interval::Interval;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use clap::Parser;
use log::{error, info};
use time::OffsetDateTime;
use uuid::Uuid;

#[derive(Parser)]
pub struct CommandPush {
    /// Project name
    #[arg(short, long)]
    project: Vec<Identifier>,
    /// Short name for the activity
    #[arg(short, long)]
    name: Option<String>,
    /// Description of the activity
    #[arg(short, long)]
    description: Option<String>,
    /// Classification of the activity
    #[arg(short, long = "class")]
    classification: Identifier,
    /// Start the activity at a specific time (HH:MM or HH:MM:SS)
    #[arg(long, conflicts_with = "offset")]
    at: Option<TimeAt>,
    /// Start the activity offset from now (e.g. -5m, +1h30m, -90s)
    #[arg(long)]
    offset: Option<TimeOffset>,
}

impl ExecutableCommand for CommandPush {
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

        if job_config.resolve_class(&self.classification).is_none() {
            error!(
                "Failed to resolve classification: {:?}",
                self.classification
            );
            return Err(std::io::Error::other(
                "Failed to resolve classification",
            ));
        }

        self
            .project
            .iter()
            .map(|id| if let Some(p) = job_config.resolve_project(id) { Ok(p) } else {
                error!("Failed to resolve project: {id:?}");
                Err(std::io::Error::other(
                    "Failed to resolve project",
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        let today = manager.get_or_create_day_mut(today_date);

        let start = resolve_time(self.at, self.offset);
        let activity = Activity {
            id: Uuid::new_v4(),
            class: self.classification.clone(),
            name: self.name.clone(),
            projects: self.project.clone(),
            time: Interval::start_at(start),
        };

        info!("Pushing new activity: {activity}");

        if config.json {
            let json = activity_json(&activity, today_date, job_config, None);
            today.activities.push(activity);
            print_json(&json);
        } else {
            println!("Started: {activity}");
            today.activities.push(activity);
        }

        Ok(())
    }
}
