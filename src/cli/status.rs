use crate::az_hash::AZHash;
use crate::cli::ExecutableCommand;
use crate::cli::hash::{all_hashes, render_hash, stdout_color, unique_prefix_map};
use crate::cli::json_output::{activity_json, print_json};
use crate::data::activity::Activity;
use crate::data::app_config::AppConfig;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use clap::Parser;
use itertools::Itertools;
use log::error;
use serde_json::json;
use std::borrow::Borrow;
use time::{Duration, OffsetDateTime};

fn format_duration_pretty<Q: Borrow<Duration>>(duration: Q, show_seconds: bool) -> String {
    let duration = duration.borrow();

    let sign = if duration.is_negative() { "-" } else { "" };

    let hours = duration.whole_hours().abs();
    let minutes = (duration.whole_minutes() % 60).abs();
    let seconds = (duration.whole_seconds() % 60).abs();

    let hours = if hours > 0 {
        format!("{hours}h ")
    } else {
        String::new()
    };
    let minutes = if minutes > 0 || !hours.is_empty() {
        format!("{minutes}m ")
    } else {
        String::new()
    };
    let seconds = if show_seconds && (seconds > 0 || !minutes.is_empty() || !hours.is_empty()) {
        format!("{seconds}s")
    } else {
        String::new()
    };

    format!("{sign}{hours}{minutes}{seconds}")
}

#[derive(Parser, Default, Clone)]
pub struct CommandStatus {}

impl ExecutableCommand for CommandStatus {
    type Error = std::io::Error;
    type Output = ();
    #[allow(clippy::too_many_lines)]
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

        // Compute globally unique hash prefixes before the mutable borrow
        // from get_or_create_day_ref so the borrow checker is happy.
        let hashes = all_hashes(&manager);
        let umap = unique_prefix_map(&hashes);
        let color = stdout_color();

        let today = manager.get_or_create_day_ref(today_date);

        if today.activities.is_empty() {
            if config.json {
                print_json(&json!({ "date": today_date.to_string(), "activities": serde_json::Value::Array(vec![]), "total_seconds": 0 }));
            } else {
                println!("No activities for today.");
            }
        } else {
            let now = OffsetDateTime::now_local().unwrap_or_else(|e| {
                error!("Failed to get local time. Falling back to UTC: {e}");
                OffsetDateTime::now_utc()
            });

            let folded = Activity::calculate_activity_closure(
                job_config,
                &today.activities,
                None,
                Some(now.time()),
            );

            // Aligned hash display: max unique prefix (floor 4) + 2 extra.
            let max_ulen = today.activities.iter()
                .map(|a| { let h = a.az_hash_sha512(); *umap.get(&h).unwrap_or(&1) })
                .max()
                .unwrap_or(1)
                .max(4);
            let total_len = max_ulen + 2;

            let hash_tag = |activity: &Activity| -> String {
                let h = activity.az_hash_sha512();
                let ulen = *umap.get(&h).unwrap_or(&1);
                render_hash(&h, ulen, total_len, color)
            };

            // Resolve class names up front for column width.
            let resolve_class_name = |activity: &Activity| -> String {
                if let Some(c) = job_config.resolve_class(&activity.class) { c.inner.name.clone() } else {
                    error!("Failed to resolve class with id {}", activity.class);
                    "ERR".to_string()
                }
            };

            let ended: Vec<Activity> = today.activities.iter()
                .filter(|a| a.time.is_complete())
                .cloned()
                .collect_vec();
            let ongoing: Vec<Activity> = today.activities.iter()
                .filter(|a| !a.time.is_complete())
                .cloned()
                .collect_vec();

            let total_seconds: i64 = folded.iter()
                .map(|a| a.time.duration().unwrap_or_default().whole_seconds())
                .sum();

            let effective_status = Activity::fold_inner(job_config, ongoing.iter(), None, None)
                .and_then(|s| job_config.resolve_class(&s.class).map(|c| c.inner.name.clone()));

            if config.json {
                let act_json: Vec<_> = today.activities.iter()
                    .map(|a| {
                        let h = a.az_hash_sha512();
                        let ulen = *umap.get(&h).unwrap_or(&1);
                        activity_json(a, today_date, job_config, Some(ulen))
                    })
                    .collect();
                print_json(&json!({
                    "date": today_date.to_string(),
                    "status": effective_status,
                    "total_seconds": total_seconds,
                    "activities": act_json,
                }));
            } else {
                let class_w = today.activities.iter()
                    .map(|a| resolve_class_name(a).len())
                    .max()
                    .unwrap_or(1);

                match &effective_status {
                    Some(name) => println!("Status: {name}"),
                    None if ongoing.is_empty() => println!("Status: idle"),
                    None => { error!("Failed to compute status."); println!("Status: ERR"); }
                }

                println!();
                for activity in &folded {
                    println!("  {activity}");
                }
                println!(
                    "\nTotal tracked: {}",
                    format_duration_pretty(
                        folded.iter()
                            .map(|a| a.time.duration().unwrap_or_default())
                            .sum::<Duration>(),
                        true,
                    )
                );

                if !ongoing.is_empty() {
                    println!("\nOngoing:");
                    for activity in &ongoing {
                        let class = resolve_class_name(activity);
                        println!(
                            "  {} [{:<class_w$}]  {}",
                            hash_tag(activity), class, activity,
                            class_w = class_w,
                        );
                    }
                }

                if !ended.is_empty() {
                    println!("\nEnded:");
                    for activity in &ended {
                        let class = resolve_class_name(activity);
                        println!(
                            "  {} [{:<class_w$}]  {}",
                            hash_tag(activity), class, activity,
                            class_w = class_w,
                        );
                    }
                }
            }
        }

        Ok(())
    }
}
