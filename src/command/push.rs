use crate::command::ExecutableCommand;
use clap::Parser;
use log::{error, info};
use time::OffsetDateTime;
use timetrax::data::activity::Activity;
use timetrax::data::app_config::AppConfig;
use timetrax::data::identifier::Identifier;
use timetrax::data::interval::Interval;
use timetrax::data::job_config::JobConfig;
use timetrax::data::manager::Manager;
use uuid::Uuid;

#[derive(Parser)]
pub struct CommandPush {
    /// Project name
    #[arg(short, long)]
    project: Vec<Identifier>,
    /// Short name for the activity
    #[arg(short, long)]
    named: Option<String>,
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
            name: self.named.clone(),
            projects: self.project.clone(),
            time: Interval::start_now(),
        };

        info!("Pushing new activity: {activity}");
        today.activities.push(activity);

        Ok(())
    }
}
