use crate::command::ExecutableCommand;
use clap::Parser;
use itertools::Itertools;
use log::error;
use std::borrow::Borrow;
use time::{Duration, OffsetDateTime};
use timetrax::data::activity::Activity;
use timetrax::data::app_config::AppConfig;
use timetrax::data::job_config::JobConfig;
use timetrax::data::manager::Manager;

fn format_duration_pretty<Q: Borrow<Duration>>(duration: Q, show_seconds: bool) -> String {
    let duration = duration.borrow();

    let sign = if duration.is_negative() { "-" } else { "" };

    let hours = duration.whole_hours().abs();
    let minutes = (duration.whole_minutes() % 60).abs();
    let seconds = (duration.whole_seconds() % 60).abs();

    let hours = if hours > 0 {
        format!("{}h ", hours)
    } else {
        "".to_string()
    };
    let minutes = if minutes > 0 || hours.len() > 0 {
        format!("{}m ", minutes)
    } else {
        "".to_string()
    };
    let seconds = if show_seconds && (seconds > 0 || minutes.len() > 0 || hours.len() > 0) {
        format!("{}s", seconds)
    } else {
        "".to_string()
    };

    format!("{sign}{hours}{minutes}{seconds}")
}

#[derive(Parser, Default, Clone)]
pub struct CommandStatus {}

impl ExecutableCommand for CommandStatus {
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

        let today = manager.get_or_create_day_ref(today);

        if today.activities.is_empty() {
            println!("No activities for today.");
        } else {
            let now = OffsetDateTime::now_local().unwrap_or_else(|e| {
                error!("Failed to get local time. Falling back to UTC: {}", e);
                OffsetDateTime::now_utc()
            });

            let folded = Activity::calculate_activity_closure(
                job_config,
                &today.activities,
                None,
                Some(now.time()),
            );
            for activity in &folded {
                println!(" --> {}", activity);
            }
            println!(
                "Total time tracked today: {}",
                format_duration_pretty(
                    folded
                        .iter()
                        .map(|a| a.time.duration().unwrap_or_default())
                        .sum::<Duration>(),
                    true
                )
            );

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
                let status = Activity::fold_inner(job_config, ongoing.iter(), None, None);
                if let Some(status) = status {
                    if let Some(class) = job_config.resolve_class(&status.class) {
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
                    let class = match job_config.resolve_class(&activity.class) {
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
                    let class = match job_config.resolve_class(&activity.class) {
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
