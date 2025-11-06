use crate::cli::ExecutableCommand;
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
}

impl ExecutableCommand for CommandPush {
    type Error = std::io::Error;
    type Output = ();
    fn execute(
        &self,
        _config: &AppConfig,
        job_config: &mut JobConfig,
        mut manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        let today = OffsetDateTime::now_local()
            .unwrap_or_else(|e| {
                error!("Failed to get local time. Falling back to UTC: {}", e);
                OffsetDateTime::now_utc()
            })
            .date();

        if let None = job_config.resolve_class(&self.classification) {
            error!(
                "Failed to resolve classification: {:?}",
                self.classification
            );
            return Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                "Failed to resolve classification",
            ));
        };

        if let Err(err) = self
            .project
            .iter()
            .map(|id| match job_config.resolve_project(id) {
                Some(p) => Ok(p),
                None => {
                    error!("Failed to resolve project: {:?}", id);
                    Err(std::io::Error::new(
                        std::io::ErrorKind::Other,
                        "Failed to resolve project",
                    ))
                }
            })
            .collect::<Result<Vec<_>, _>>()
        {
            return Err(err);
        }

        let today = manager.get_or_create_day_mut(today);

        let activity = Activity {
            id: Uuid::new_v4(),
            class: self.classification.clone(),
            name: self.name.clone(),
            projects: self.project.clone(),
            time: Interval::start_now(),
        };

        info!("Pushing new activity: {activity}");
        today.activities.push(activity);

        Ok(())
    }
}
