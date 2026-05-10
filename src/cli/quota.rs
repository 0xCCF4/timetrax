use crate::cli::ExecutableCommand;
use crate::cli::json_output::print_json;
use crate::cli::time_input::{CliDuration, WeekdayInput, fmt_duration, weekday_name};
use crate::data::app_config::AppConfig;
use crate::data::identifier::Identifier;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use crate::data::week_quota::ClassQuota;
use clap::Parser;
use log::error;
use serde_json::json;
use time::Weekday;

const ALL_WEEKDAYS: [Weekday; 7] = [
    Weekday::Monday,
    Weekday::Tuesday,
    Weekday::Wednesday,
    Weekday::Thursday,
    Weekday::Friday,
    Weekday::Saturday,
    Weekday::Sunday,
];

#[derive(Parser)]
pub enum CommandQuota {
    /// List expected working times for each weekday
    #[clap(aliases = ["ls", "show"])]
    List,
    /// Set expected duration for an activity class on a weekday
    #[clap(aliases = ["add", "update"])]
    Set {
        /// Weekday (mon, tue, wed, thu, fri, sat, sun)
        day: WeekdayInput,
        /// Activity class identifier
        class: Identifier,
        /// Expected duration (e.g. 8h, 8h30m, 45m)
        duration: CliDuration,
    },
    /// Remove expected duration for an activity class on a weekday
    #[clap(aliases = ["delete", "del", "rm"])]
    Remove {
        /// Weekday (mon, tue, wed, thu, fri, sat, sun)
        day: WeekdayInput,
        /// Activity class identifier
        class: Identifier,
    },
}

impl ExecutableCommand for CommandQuota {
    type Error = std::io::Error;
    type Output = ();
    fn execute(
        &self,
        config: &AppConfig,
        job_config: &mut JobConfig,
        _manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        match self {
            CommandQuota::List => {
                if config.json {
                    let mut obj = serde_json::Map::new();
                    for &wd in &ALL_WEEKDAYS {
                        let entries: Vec<_> = job_config.week_quotas.for_weekday(wd).iter().map(|q| {
                            let class_name = job_config.resolve_class(&q.class).map_or_else(|| q.class.to_string(), |c| c.inner.name.clone());
                            json!({ "class": class_name, "duration_seconds": q.duration.whole_seconds() })
                        }).collect();
                        obj.insert(weekday_name(wd).to_lowercase(), json!(entries));
                    }
                    print_json(&obj);
                } else {
                    let any = ALL_WEEKDAYS
                        .iter()
                        .any(|&wd| !job_config.week_quotas.for_weekday(wd).is_empty());
                    if !any {
                        println!("No week quotas configured.");
                        return Ok(());
                    }
                    for &wd in &ALL_WEEKDAYS {
                        let entries = job_config.week_quotas.for_weekday(wd);
                        if entries.is_empty() { continue; }
                        println!("{}:", weekday_name(wd));
                        for q in entries {
                            let class_name = job_config.resolve_class(&q.class).map_or_else(|| q.class.as_str_repr(), |c| c.inner.name.as_str());
                            println!("  {:<16} {}", class_name, fmt_duration(q.duration));
                        }
                    }
                }
            }

            CommandQuota::Set { day, class, duration } => {
                if job_config.resolve_class(class).is_none() {
                    error!("Failed to resolve class: {class:?}");
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "activity class not found",
                    ));
                }
                let entries = job_config.week_quotas.for_weekday_mut(day.0);
                if let Some(existing) = entries.iter_mut().find(|q| q.class == *class) {
                    existing.duration = duration.0;
                } else {
                    entries.push(ClassQuota { class: class.clone(), duration: duration.0 });
                }
                if config.json {
                    print_json(&json!({ "day": weekday_name(day.0), "class": class.to_string(), "duration_seconds": duration.0.whole_seconds() }));
                } else {
                    println!("Set quota for {} on {}: {}", class, weekday_name(day.0), fmt_duration(duration.0));
                }
            }

            CommandQuota::Remove { day, class } => {
                let entries = job_config.week_quotas.for_weekday_mut(day.0);
                let before = entries.len();
                entries.retain(|q| q.class != *class);
                if entries.len() == before {
                    error!("No quota found for {:?} on {}", class, weekday_name(day.0));
                    return Err(std::io::Error::new(
                        std::io::ErrorKind::NotFound,
                        "quota not found",
                    ));
                }
                if config.json {
                    print_json(&json!({ "removed": true, "day": weekday_name(day.0), "class": class.to_string() }));
                } else {
                    println!("Removed quota for {} on {}", class, weekday_name(day.0));
                }
            }
        }
        Ok(())
    }
}
