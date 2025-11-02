use crate::command::ExecutableCommand;
use clap::Parser;
use itertools::Itertools;
use log::error;
use time::OffsetDateTime;
use timetrax::data::activity::Activity;
use timetrax::data::app_config::AppConfig;
use timetrax::data::manager::Manager;

#[derive(Parser)]
pub struct CommandStatus {}

impl ExecutableCommand for CommandStatus {
    type Error = std::io::Error;
    type Output = ();
    fn execute(
        &self,
        config: &AppConfig,
        mut manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        let today = OffsetDateTime::now_local()
            .unwrap_or_else(|e| {
                error!("Failed to get local time. Falling back to UTC: {}", e);
                OffsetDateTime::now_utc()
            })
            .date();

        let today = manager.get_or_create_day_ref(today);

        if today.activities.is_empty() {
            println!("No activities for today.");
        } else {
            let ended = today
                .activities
                .iter()
                .filter(|a| a.time.is_complete())
                .cloned()
                .collect_vec();
            let ongoing = today
                .activities
                .iter()
                .filter(|a| !a.time.is_complete())
                .cloned()
                .collect_vec();

            if !ongoing.is_empty() {
                let status = Activity::fold_inner(manager.job_config(), ongoing.iter(), None, None);
                if let Some(status) = status {
                    if let Some(class) = manager.job_config().resolve_class(&status.class) {
                        println!("Status: {}", class.inner.name);
                    } else {
                        error!("Failed to resolve class with id {}", status.class);
                        println!("Status: ERR");
                    }
                } else {
                    error!("Failed to compute status.");
                    println!("Status: ERR");
                }

                println!("Ongoing activities:");
                for activity in ongoing {
                    let class = match manager.job_config().resolve_class(&activity.class) {
                        Some(class) => class.inner.name.as_str(),
                        None => {
                            error!("Failed to resolve class with id {}", activity.class);
                            "ERR"
                        }
                    };
                    println!(" - [{}] {}", class, activity);
                }
            } else {
                println!("No ongoing activities.");
            }

            if !ended.is_empty() {
                println!("Ended activities:");
                for activity in ended {
                    let class = match manager.job_config().resolve_class(&activity.class) {
                        Some(class) => class.inner.name.as_str(),
                        None => {
                            error!("Failed to resolve class with id {}", activity.class);
                            "ERR"
                        }
                    };
                    println!(" - [{}] {}", class, activity);
                }
            } else {
                println!("No ended activities.");
            }
        }

        Ok(())
    }
}
