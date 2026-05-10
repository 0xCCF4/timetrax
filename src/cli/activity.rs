use crate::az_hash::AZHash;
use crate::cli::ExecutableCommand;
use crate::cli::hash::{all_hashes, render_hash, stderr_color, stdout_color, unique_prefix_map};
use crate::cli::json_output::{activity_json, print_json};
use crate::cli::time_input::{DateInput, TimeAt};
use crate::data::app_config::AppConfig;
use crate::data::identifier::Identifier;
use crate::data::job_config::JobConfig;
use crate::data::manager::Manager;
use clap::Parser;
use log::error;
use serde_json::json;
use time::{Date, OffsetDateTime};
use uuid::Uuid;

/// Scan all days and return the `(date, activity_uuid)` that uniquely matches
/// `prefix`.  On ambiguity the error message shows each match's highlighted
/// unique prefix so the user knows how much more to type.
fn resolve_prefix(manager: &Manager, prefix: &str) -> std::io::Result<(Date, Uuid)> {
    let prefix = prefix.to_lowercase();
    let mut matches: Vec<(Date, Uuid, String)> = Vec::new();

    for (date, day_info) in &manager.days {
        for activity in &day_info.inner().activities {
            let hash = activity.az_hash_sha512();
            if hash.starts_with(&prefix) {
                matches.push((*date, activity.id, hash));
            }
        }
    }

    match matches.len() {
        0 => {
            error!("No activity matches prefix '{prefix}'");
            Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("no activity matches prefix '{prefix}'"),
            ))
        }
        1 => Ok((matches[0].0, matches[0].1)),
        n => {
            let color = stderr_color();
            let hashes: Vec<String> = matches.iter().map(|(_, _, h)| h.clone()).collect();
            let umap = unique_prefix_map(&hashes);
            eprintln!("Prefix '{prefix}' is ambiguous - {n} activities match:");
            for (date, _, hash) in &matches {
                let ulen = *umap.get(hash).unwrap_or(&prefix.len());
                let rendered = render_hash(hash, ulen, ulen + 4, color);
                eprintln!("  {date}  {rendered}");
            }
            Err(std::io::Error::other(
                format!("ambiguous prefix '{prefix}': {n} matches"),
            ))
        }
    }
}

#[derive(Parser)]
pub enum CommandActivity {
    /// List activities with their hash identifiers
    #[clap(aliases = ["ls", "show"])]
    List {
        /// Only show activities for this date (YYYY-MM-DD); defaults to today
        #[arg(long, short, conflicts_with_all = ["all", "from", "to"])]
        date: Option<DateInput>,
        /// Show activities for all dates in the database
        #[arg(long, short, conflicts_with_all = ["date", "from", "to"])]
        all: bool,
        /// Start of date range, inclusive (YYYY-MM-DD)
        #[arg(long, conflicts_with = "date")]
        from: Option<DateInput>,
        /// End of date range, inclusive (YYYY-MM-DD); defaults to today when --from is given
        #[arg(long, conflicts_with = "date")]
        to: Option<DateInput>,
    },
    /// Edit an existing activity's fields
    #[clap(aliases = ["e", "modify"])]
    Edit {
        /// Unique hash prefix identifying the activity
        prefix: String,
        /// Set or update the activity name
        #[arg(long)]
        name: Option<String>,
        /// Remove the activity name
        #[arg(long, conflicts_with = "name")]
        no_name: bool,
        /// Change the activity class
        #[arg(long = "class", short = 'c')]
        classification: Option<Identifier>,
        /// Change the start time (HH:MM or HH:MM:SS)
        #[arg(long)]
        start: Option<TimeAt>,
        /// Change the end time (HH:MM or HH:MM:SS)
        #[arg(long, conflicts_with = "no_end")]
        end: Option<TimeAt>,
        /// Remove the end time, re-opening the activity
        #[arg(long)]
        no_end: bool,
        /// Add a project to this activity
        #[arg(long)]
        add_project: Vec<Identifier>,
        /// Remove a project from this activity
        #[arg(long)]
        remove_project: Vec<Identifier>,
    },
    /// Remove an activity from the database
    #[clap(aliases = ["rm", "delete", "del"])]
    Remove {
        /// Unique hash prefix identifying the activity (conflicts with --name)
        #[arg(conflicts_with = "name", required_unless_present = "name")]
        prefix: Option<String>,
        /// Remove activity by exact name instead of hash prefix
        #[arg(long, conflicts_with = "prefix")]
        name: Option<String>,
        /// Date to search when using --name (YYYY-MM-DD); defaults to today
        #[arg(long, short)]
        date: Option<DateInput>,
    },
}

enum ListFilter { All, Single(Date), Range(Option<Date>, Option<Date>) }

struct ListRow {
    date: Date,
    hash: String,
    ulen: usize,
    class: String,
    activity_str: String,
}

impl ExecutableCommand for CommandActivity {
    type Error = std::io::Error;
    type Output = ();

    #[allow(clippy::too_many_lines)]
    fn execute(
        &self,
        config: &AppConfig,
        job_config: &mut JobConfig,
        mut manager: Manager,
    ) -> Result<Self::Output, Self::Error> {
        match self {
            CommandActivity::List { date, all, from, to } => {
                let today = OffsetDateTime::now_local()
                    .unwrap_or_else(|e| {
                        error!("Failed to get local time. Falling back to UTC: {e}");
                        OffsetDateTime::now_utc()
                    })
                    .date();

                let filter = if *all {
                    ListFilter::All
                } else if from.is_some() || to.is_some() {
                    ListFilter::Range(from.map(|d| d.0), to.map(|d| d.0))
                } else {
                    ListFilter::Single(date.map_or(today, |d| d.0))
                };

                // Uniqueness computed globally so prefixes are valid for edit/remove.
                let hashes = all_hashes(&manager);
                let umap = unique_prefix_map(&hashes);
                let color = stdout_color();

                // First pass: collect visible rows to compute aligned column width.
                let mut rows: Vec<ListRow> = Vec::new();

                for (day_date, day_info) in &manager.days {
                    let include = match &filter {
                        ListFilter::All => true,
                        ListFilter::Single(d) => day_date == d,
                        ListFilter::Range(f, t) => {
                            f.is_none_or(|f| *day_date >= f) && t.is_none_or(|t| *day_date <= t)
                        }
                    };
                    if !include { continue; }
                    for activity in &day_info.inner().activities {
                        let hash = activity.az_hash_sha512();
                        let ulen = *umap.get(&hash).unwrap_or(&1);
                        let class = job_config
                            .resolve_class(&activity.class).map_or_else(|| "?".to_string(), |c| c.inner.name.clone());
                        rows.push(ListRow {
                            date: *day_date,
                            hash,
                            ulen,
                            class,
                            activity_str: activity.to_string(),
                        });
                    }
                }

                if rows.is_empty() {
                    if config.json {
                        print_json(&serde_json::Value::Array(vec![]));
                    } else {
                        println!("No activities found.");
                    }
                } else if config.json {
                    // For JSON we need Activity refs - re-iterate manager with umap
                    let json_rows: Vec<_> = manager.days.iter()
                        .flat_map(|(day_date, day_info)| {
                            let include = match &filter {
                                ListFilter::All => true,
                                ListFilter::Single(d) => day_date == d,
                                ListFilter::Range(f, t) => {
                                    f.is_none_or(|f| *day_date >= f) && t.is_none_or(|t| *day_date <= t)
                                }
                            };
                            if !include { return vec![]; }
                            day_info.inner().activities.iter().map(|a| {
                                let h = a.az_hash_sha512();
                                let ulen = *umap.get(&h).unwrap_or(&1);
                                activity_json(a, *day_date, job_config, Some(ulen))
                            }).collect::<Vec<_>>()
                        })
                        .collect();
                    print_json(&json_rows);
                } else {
                    // Align: max unique prefix (floor 4) + 2 extra chars shown for all.
                    let max_ulen = rows.iter().map(|r| r.ulen).max().unwrap_or(1).max(4);
                    let total_len = max_ulen + 2;
                    let class_w = rows.iter().map(|r| r.class.len()).max().unwrap_or(1);

                    for r in &rows {
                        let rendered = render_hash(&r.hash, r.ulen, total_len, color);
                        println!(
                            "{}  {}  [{:<class_w$}]  {}",
                            r.date, rendered, r.class, r.activity_str,
                            class_w = class_w,
                        );
                    }
                }
            }

            CommandActivity::Edit {
                prefix,
                name,
                no_name,
                classification,
                start,
                end,
                no_end,
                add_project,
                remove_project,
            } => {
                let (date, uuid) = resolve_prefix(&manager, prefix)?;

                if let Some(cls) = classification
                    && job_config.resolve_class(cls).is_none() {
                        error!("Class not found: {cls}");
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("class '{cls}' not found"),
                        ));
                    }
                for proj in add_project.iter().chain(remove_project.iter()) {
                    if job_config.resolve_project(proj).is_none() {
                        error!("Project not found: {proj}");
                        return Err(std::io::Error::new(
                            std::io::ErrorKind::NotFound,
                            format!("project '{proj}' not found"),
                        ));
                    }
                }

                let day = manager.get_or_create_day(date).inner_mut();
                let activity = day
                    .activities
                    .iter_mut()
                    .find(|a| a.id == uuid)
                    .expect("activity disappeared between lookup and edit");

                if *no_name {
                    activity.name = None;
                } else if let Some(n) = name {
                    activity.name = Some(n.clone());
                }
                if let Some(cls) = classification {
                    activity.class = cls.clone();
                }
                if let Some(s) = start {
                    activity.time.start = s.0;
                }
                if *no_end {
                    activity.time.end = None;
                } else if let Some(e) = end {
                    activity.time.end = Some(e.0);
                }
                for proj in add_project {
                    if !activity.projects.contains(proj) {
                        activity.projects.push(proj.clone());
                    }
                }
                activity.projects.retain(|p| !remove_project.contains(p));

                if config.json {
                    let json = activity_json(activity, date, job_config, None);
                    print_json(&json);
                } else {
                    println!("Updated activity: {activity}");
                }
            }

            CommandActivity::Remove { prefix, name, date } => {
                if let Some(name) = name {
                    let today = OffsetDateTime::now_local()
                        .unwrap_or_else(|e| {
                            error!("Failed to get local time. Falling back to UTC: {e}");
                            OffsetDateTime::now_utc()
                        })
                        .date();
                    let target_date = date.map_or(today, |d| d.0);

                    let day = manager.get_or_create_day(target_date).inner_mut();
                    let before = day.activities.len();
                    day.activities.retain(|a| a.name.as_deref() != Some(name.as_str()));
                    let removed = before - day.activities.len();

                    match removed {
                        0 => {
                            error!("No activity named '{name}' found on {target_date}");
                            return Err(std::io::Error::new(
                                std::io::ErrorKind::NotFound,
                                format!("no activity named '{name}' on {target_date}"),
                            ));
                        }
                        n => {
                            if config.json {
                                print_json(&json!({ "removed": n, "name": name, "date": target_date.to_string() }));
                            } else if n == 1 {
                                println!("Removed activity '{name}' from {target_date}");
                            } else {
                                println!("Removed {n} activities named '{name}' from {target_date}");
                            }
                        }
                    }
                } else {
                    let prefix = prefix.as_deref().expect("prefix is required when --name is absent");
                    let (date, uuid) = resolve_prefix(&manager, prefix)?;

                    let day = manager.get_or_create_day(date).inner_mut();
                    day.activities.retain(|a| a.id != uuid);

                    if config.json {
                        print_json(&json!({ "removed": 1, "date": date.to_string(), "id": uuid.to_string() }));
                    } else {
                        println!("Removed activity from {date}");
                    }
                }
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::activity::Activity;
    use crate::data::day::DayInner;
    use crate::data::dirty::DirtyMarker;
    use crate::data::interval::Interval;
    use crate::data::manager::{AnnotatedDayInformation, Manager};
    use std::collections::BTreeMap;
    use std::path::PathBuf;
    use time::Time;
    use uuid::Uuid;

    fn make_config() -> AppConfig {
        AppConfig::default()
    }

    fn make_activity_named(name: &str, start: u8, end: u8) -> Activity {
        Activity {
            id: Uuid::new_v4(),
            name: Some(name.into()),
            class: Identifier::ByName("work".into()),
            time: Interval {
                start: Time::from_hms(start, 0, 0).unwrap(),
                end: Some(Time::from_hms(end, 0, 0).unwrap()),
            },
            projects: vec![],
        }
    }

    fn make_day_with_activities(activities: Vec<Activity>) -> DirtyMarker<DayInner> {
        DirtyMarker::from(DayInner { activities, ..DayInner::default() })
    }

    fn manager_with_days(
        config: &AppConfig,
        days: Vec<(time::Date, Vec<Activity>)>,
    ) -> Manager<'_> {
        let mut map = BTreeMap::new();
        for (date, acts) in days {
            map.insert(
                date,
                AnnotatedDayInformation::Unsaved {
                    day: make_day_with_activities(acts),
                },
            );
        }
        Manager { app_config: config, days: map, data_path: PathBuf::new() }
    }

    fn date(y: i32, m: u8, d: u8) -> time::Date {
        time::Date::from_calendar_date(y, time::Month::try_from(m).unwrap(), d).unwrap()
    }

    #[test]
    fn unique_prefix_single_hash_uses_minimum() {
        let hashes = vec!["abcdefgh".to_string()];
        let map = unique_prefix_map(&hashes);
        assert_eq!(map["abcdefgh"], 1);
    }

    #[test]
    fn unique_prefix_identical_prefix_forces_longer() {
        let hashes = vec!["abcdefgh".to_string(), "abcxyz12".to_string()];
        let map = unique_prefix_map(&hashes);
        // first 3 chars ("abc") are shared, so need 4 chars
        assert_eq!(map["abcdefgh"], 4);
        assert_eq!(map["abcxyz12"], 4);
    }

    #[test]
    fn unique_prefix_long_shared_prefix() {
        let hashes = vec!["abcde111".to_string(), "abcde222".to_string()];
        let map = unique_prefix_map(&hashes);
        // first 5 chars shared → need 6
        assert_eq!(map["abcde111"], 6);
        assert_eq!(map["abcde222"], 6);
    }

    #[test]
    fn unique_prefix_three_hashes_independent() {
        let hashes = vec![
            "aaaaaaaa".to_string(),
            "bbbbbbbb".to_string(),
            "cccccccc".to_string(),
        ];
        let map = unique_prefix_map(&hashes);
        // all diverge at char 0 → need 1, but clamped to MIN_UNIQUE=4
        assert_eq!(map["aaaaaaaa"], 1);
        assert_eq!(map["bbbbbbbb"], 1);
        assert_eq!(map["cccccccc"], 1);
    }

    #[test]
    fn render_hash_plain_splits_at_unique_len() {
        let rendered = render_hash("abcdefgh", 3, 7, false);
        assert_eq!(rendered, "abcdefg"); // 3 unique + 4 dim = 7 chars
    }

    #[test]
    fn render_hash_color_wraps_segments() {
        let rendered = render_hash("abcdefgh", 3, 7, true);
        assert!(rendered.starts_with("\x1b[1mabc\x1b[0m"));
        assert!(rendered.contains("\x1b[2mdefg\x1b[0m"));
    }

    #[test]
    fn render_hash_unique_len_equals_total() {
        // dim portion is empty
        let rendered = render_hash("abcd", 4, 4, false);
        assert_eq!(rendered, "abcd");
    }

    #[test]
    fn render_hash_clamps_to_hash_length() {
        let rendered = render_hash("ab", 4, 10, false);
        assert_eq!(rendered, "ab"); // hash shorter than requested total
    }

    #[test]
    fn list_filter_range_includes_exact_bounds() {
        let config = make_config();
        let d1 = date(2024, 1, 10);
        let d2 = date(2024, 1, 15);
        let d3 = date(2024, 1, 20);
        let manager = manager_with_days(&config, vec![
            (d1, vec![make_activity_named("early", 9, 10)]),
            (d2, vec![make_activity_named("mid", 9, 10)]),
            (d3, vec![make_activity_named("late", 9, 10)]),
        ]);
        let filter = ListFilter::Range(Some(d1), Some(d2));
        let visible: Vec<_> = manager.days.iter()
            .filter(|(day_date, _)| match filter {
                ListFilter::Range(f, t) => {
                    f.is_none_or(|f| **day_date >= f) && t.is_none_or(|t| **day_date <= t)
                }
                _ => false,
            })
            .collect();
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn list_filter_range_open_end_includes_all_after_start() {
        let config = make_config();
        let d1 = date(2024, 1, 10);
        let d2 = date(2024, 1, 20);
        let manager = manager_with_days(&config, vec![
            (d1, vec![make_activity_named("a", 9, 10)]),
            (d2, vec![make_activity_named("b", 9, 10)]),
        ]);
        let filter = ListFilter::Range(Some(d1), None);
        let visible: Vec<_> = manager.days.iter()
            .filter(|(day_date, _)| match filter {
                ListFilter::Range(f, t) => {
                    f.is_none_or(|f| **day_date >= f) && t.is_none_or(|t| **day_date <= t)
                }
                _ => false,
            })
            .collect();
        assert_eq!(visible.len(), 2);
    }

    #[test]
    fn remove_name_counts_removed_activities() {
        let a1 = make_activity_named("standup", 9, 10);
        let a2 = make_activity_named("standup", 14, 15);
        let a3 = make_activity_named("other", 11, 12);
        let mut day = DayInner { activities: vec![a1, a2, a3], ..DayInner::default() };

        let before = day.activities.len();
        let name = "standup";
        day.activities.retain(|a| a.name.as_deref() != Some(name));
        let removed = before - day.activities.len();

        assert_eq!(removed, 2);
        assert_eq!(day.activities.len(), 1);
        assert_eq!(day.activities[0].name.as_deref(), Some("other"));
    }

    #[test]
    fn remove_name_zero_when_not_found() {
        let a1 = make_activity_named("other", 9, 10);
        let mut day = DayInner { activities: vec![a1], ..DayInner::default() };

        let before = day.activities.len();
        day.activities.retain(|a| a.name.as_deref() != Some("missing"));
        let removed = before - day.activities.len();

        assert_eq!(removed, 0);
        assert_eq!(day.activities.len(), 1);
    }
}
