use crate::az_hash::AZHash;
use itertools::Itertools;
use log::{error, trace};
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::collections::BinaryHeap;
use std::fmt::{Display, Formatter};
use std::sync::LazyLock;
use time::{Time, UtcDateTime, format_description};
use uuid::Uuid;

/// Specified time interval, may be open-ended
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Interval {
    pub start: time::Time,
    pub end: Option<time::Time>,
}

impl Interval {
    /// Duration if interval ended
    pub fn duration(&self) -> Option<time::Duration> {
        if let Some(end_time) = self.end {
            Some(end_time - self.start)
        } else {
            None
        }
    }

    /// Interval completed
    pub fn is_complete(&self) -> bool {
        self.end.is_some()
    }

    /// end of interval or if open-ended, end of day
    pub fn end_time_or_end_of_day(&self) -> time::Time {
        self.end.unwrap_or(time::Time::MAX)
    }
}
/// Activity class, defines the types of activities
/// the order defines the override order, see `Activity`
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq, Hash, Ord, PartialOrd)]
pub enum ActivityClass {
    /// Time worked
    Work,
    /// Break time
    Break,
    /// Excused due to some reason (e.g. doctor's appointment), counted as work time
    Excused,
    /// Vacation days
    Holiday,
}

impl ActivityClass {
    pub fn lowest_priority() -> Self {
        ActivityClass::Work
    }
}

/// Activity
/// Multiple activities of the same class may be worked on at the same time
/// example: 9-12 work(projectA) 10-12 work(projectB)
/// An activity of type break will interrupt running WORK activities
/// An activity of type EXCUSED will interrupt running BREAK/WORK
/// An activity of type HOLIDAY will interrupt running BREAK/WORK/EXCUSED
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Activity {
    /// Unique id, used for editing reference
    pub id: Uuid,
    /// created
    #[serde(default = "time::UtcDateTime::now")]
    pub created_at: time::UtcDateTime,
    /// modified
    #[serde(default = "time::UtcDateTime::now")]
    pub modified_at: time::UtcDateTime,
    /// Optional name of the activity
    pub name: Option<String>,
    /// Activity class, work, break, ...
    pub class: ActivityClass,
    /// Time spend on the activity
    pub time: Interval,
    /// Optional description
    pub description: Option<String>,
    /// Optional tags
    pub tags: Vec<String>,
    /// Projects worked on
    pub projects: Vec<String>,
}

static TIME_FORMAT: LazyLock<Vec<format_description::BorrowedFormatItem<'_>>> =
    LazyLock::new(|| {
        time::format_description::parse(
            "[hour padding:zero]:[minute padding:zero]:[second padding:zero]",
        )
        .unwrap()
    });

impl Display for Activity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} ({:?}): {}",
            self.time.start.format(&*TIME_FORMAT).unwrap_or_else(|e| {
                error!("Unable to format time: {e}. Report this as an issue.");
                "<INVALID>".to_string()
            }),
            self.time
                .end
                .map(|t| t.format(&*TIME_FORMAT).unwrap_or_else(|e| {
                    error!("Unable to format time: {e}. Report this as an issue.");
                    "<INVALID>".to_string()
                }))
                .unwrap_or_else(|| "<OPEN>".to_string()),
            self.class,
            self.name
                .clone()
                .unwrap_or_else(|| "<NO DESCRIPTION>".to_string())
        )
    }
}

impl Activity {
    /// Folds multiple activities into a single activity
    /// the largest start time and smallest end time is used
    /// the highest priority class is used
    /// all other attributes are combined
    ///
    /// will return none if no activities are provided or the collapsed start_time > end_time
    pub fn fold_inner<Q: Borrow<Activity>, I: Iterator<Item = Q>>(
        activities: I,
        start_time_limit: Option<&Time>,
        end_time_limit: Option<&Time>,
    ) -> Option<Activity> {
        let mut start_time = None;
        let mut end_time = None;
        let mut class = ActivityClass::lowest_priority();
        let mut names = Vec::new();
        let mut descriptions = Vec::new();
        let mut tags = Vec::new();
        let mut projects = Vec::new();

        for activity in activities {
            let activity = activity.borrow();
            if start_time.is_none() || &activity.time.start > start_time.as_ref().unwrap() {
                start_time = Some(activity.time.start);
            }
            if let Some(end_time) = end_time.as_mut() {
                if let Some(activity_end) = activity.time.end {
                    if activity_end < *end_time {
                        *end_time = activity_end;
                    }
                }
            }
            {
                end_time = activity.time.end;
            }

            if activity.class > class {
                class = activity.class;
            }

            if let Some(activity_name) = &activity.name {
                if !names.contains(activity_name) {
                    names.push(activity_name.clone());
                }
            }

            if let Some(activity_description) = &activity.description {
                if !descriptions.contains(activity_description) {
                    descriptions.push(activity_description.clone());
                }
            }

            for tag in &activity.tags {
                if !tags.contains(tag) {
                    tags.push(tag.clone());
                }
            }

            for project in &activity.projects {
                if !projects.contains(project) {
                    projects.push(project.clone());
                }
            }
        }

        if let Some(mut start_time) = start_time {
            if let Some(start_time_limit) = start_time_limit {
                if start_time < *start_time_limit {
                    start_time = *start_time_limit;
                }
            }

            if let Some(end_time_limit) = end_time_limit {
                if let Some(end_time) = &mut end_time {
                    if *end_time > *end_time_limit {
                        *end_time = *end_time_limit;
                    }
                } else {
                    end_time = Some(*end_time_limit);
                }
            }

            if let Some(end_time) = end_time {
                if start_time > end_time {
                    return None;
                }
            }

            descriptions.sort();
            names.sort();
            tags.sort();
            projects.sort();

            Some(Activity {
                id: Uuid::new_v4(),
                created_at: UtcDateTime::now(),
                modified_at: UtcDateTime::now(),
                name: if names.len() == 0 {
                    None
                } else {
                    Some(names.into_iter().join("; ").into())
                },
                class,
                time: Interval {
                    start: start_time,
                    end: end_time,
                },
                description: if descriptions.len() == 0 {
                    None
                } else {
                    Some(descriptions.into_iter().join("; ").into())
                },
                tags,
                projects,
            })
        } else {
            None
        }
    }
}

impl AZHash for Activity {
    fn az_hash(&self) -> String {
        self.id.az_hash()
    }
}

/// Blocker
/// Add a constant time amount to the daily amount
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Blocker {
    /// Unique id, used for editing reference
    pub id: Uuid,
    /// created
    #[serde(default = "time::UtcDateTime::now")]
    pub created_at: time::UtcDateTime,
    /// modified
    #[serde(default = "time::UtcDateTime::now")]
    pub modified_at: time::UtcDateTime,
    /// Optional name of the activity
    pub name: Option<String>,
    /// Activity class, work, break, ...
    pub class: ActivityClass,
    /// Blocked time spend on the activity
    pub time: Interval,
    /// Optional description
    pub description: Option<String>,
    /// Optional tags
    pub tags: Vec<String>,
    /// Projects worked on
    pub projects: Vec<Uuid>,
}

impl AZHash for Blocker {
    fn az_hash(&self) -> String {
        self.id.az_hash()
    }
}

#[derive(Serialize, Deserialize)]
/// data structure for a single day
pub struct Day {
    /// date of the day
    pub date: time::Date,
    /// target work quota
    pub work_quota: time::Duration,
    /// blockers
    pub blockers: Vec<Blocker>,
    /// activities
    pub activities: Vec<Activity>,
}

impl AZHash for Day {
    fn az_hash(&self) -> String {
        self.date.to_string().az_hash()
    }
}

impl Day {
    /// Create a new day
    pub fn new(date: time::Date, work_quota: time::Duration) -> Self {
        Self {
            date,
            work_quota,
            activities: Vec::new(),
            blockers: Vec::new(),
        }
    }

    /// calculate activity closure for the day
    /// activity closure meaning a linear timeline of non-overlapping activities

    pub fn calculate_activity_closure(&self) -> Vec<Activity> {
        #[repr(transparent)]
        #[derive(Debug)]
        struct ActivitySortByEndTime(Activity);

        impl Display for ActivitySortByEndTime {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl Ord for ActivitySortByEndTime {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other
                    .0
                    .time
                    .end_time_or_end_of_day()
                    .cmp(&self.0.time.end_time_or_end_of_day())
            }
        }
        impl PartialOrd for ActivitySortByEndTime {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl PartialEq for ActivitySortByEndTime {
            fn eq(&self, other: &Self) -> bool {
                self.cmp(other) == std::cmp::Ordering::Equal
            }
        }
        impl Eq for ActivitySortByEndTime {}
        fn fold_report(
            stack: &BinaryHeap<ActivitySortByEndTime>,
            open_ended: &Vec<Activity>,
            start_time: Option<&Time>,
            end_time: Option<&Time>,
        ) -> Option<Activity> {
            let folded = Activity::fold_inner(
                stack.iter().map(|x| &x.0).chain(open_ended.iter()),
                start_time,
                end_time,
            );

            if let Some(folded) = folded {
                if folded.time.start == folded.time.end_time_or_end_of_day() {
                    return None;
                }

                Some(folded)
            } else {
                if stack.len() + open_ended.len() > 0 {
                    error!(
                        "Unable to fold activities into a single activity. Please re-run the app with debugging output and report this as a bug."
                    );
                    trace!("Activity stack:");
                    for (i, activity) in stack.iter().enumerate() {
                        trace!(
                            "  {}: {:?}-{:?}",
                            i, activity.0.time.start, activity.0.time.end
                        );
                    }
                    trace!("Open ended activities:");
                    for (i, activity) in open_ended.iter().enumerate() {
                        trace!("  {}: {:?}-{:?}", i, activity.time.start, activity.time.end);
                    }
                }
                None
            }
        }

        let mut activities: Vec<Activity> = self.activities.clone();
        // sort by start time
        activities.sort_by(|a, b| a.time.start.cmp(&b.time.start));

        let mut closure: Vec<Activity> = Vec::with_capacity(activities.len() * 2);
        let mut activity_stack: BinaryHeap<ActivitySortByEndTime> =
            BinaryHeap::with_capacity(activities.len());
        let mut open_ended_activities = Vec::with_capacity(activities.len());

        trace!("Folding activities for day {}", self.date);
        if log::max_level() >= log::LevelFilter::Trace {
            trace!("  Input activities:");
            for activity in &activities {
                trace!("   - {}", activity);
            }
        }

        let mut last_activity_end = None;
        for activity in activities.into_iter() {
            trace!(" - Processing activity: {}", activity);

            while let Some(top_activity) = activity_stack.peek() {
                trace!("   - Current activity stack:");
                if log::max_level() >= log::LevelFilter::Trace {
                    for ac in &activity_stack {
                        trace!("     {ac}");
                    }
                }

                if top_activity.0.time.end_time_or_end_of_day() <= activity.time.start {
                    trace!("   -> Dropping activity from stack: {}", top_activity);

                    if let Some(folded) = fold_report(
                        &activity_stack,
                        &open_ended_activities,
                        last_activity_end.as_ref(),
                        Some(&top_activity.0.time.end_time_or_end_of_day()),
                    ) {
                        trace!(
                            "   -> Folding current stack up to end of dropped activity: {}",
                            folded
                        );
                        closure.push(folded);
                    }

                    last_activity_end = Some(top_activity.0.time.end_time_or_end_of_day());

                    drop(activity_stack.pop());
                } else {
                    break;
                }
            }

            if let Some(folded) = fold_report(
                &activity_stack,
                &open_ended_activities,
                last_activity_end.as_ref(),
                Some(&activity.time.start),
            ) {
                trace!(
                    "   -> Folding current stack up to start of new activity: {}",
                    folded
                );
                closure.push(folded);
            }

            if activity.time.is_complete() {
                trace!("   -> Pushing activity to stack: {}", activity);
                activity_stack.push(ActivitySortByEndTime(activity));
            } else {
                trace!("   -> Adding open-ended activity: {}", activity);
                open_ended_activities.push(activity);
            }
        }

        // fold remaining activities
        trace!(" - Folding remaining activities in stack");
        if log::max_level() >= log::LevelFilter::Trace {
            trace!("   - Current activity stack:");
            for ac in &activity_stack {
                trace!("     {ac}");
            }
        }

        while let Some(activity) = activity_stack.peek() {
            trace!("   -> Dropping activity from stack: {}", activity);

            if let Some(folded) = fold_report(
                &activity_stack,
                &open_ended_activities,
                last_activity_end.as_ref(),
                activity.0.time.end.as_ref(),
            ) {
                closure.push(folded);
            }

            last_activity_end = Some(activity.0.time.end_time_or_end_of_day());

            drop(activity_stack.pop());
        }

        closure
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use time::{Time, UtcDateTime};

    #[test]
    fn test_priority_activity_classes() {
        assert!(ActivityClass::Work < ActivityClass::Break);
        assert!(ActivityClass::Break < ActivityClass::Excused);
        assert!(ActivityClass::Excused < ActivityClass::Holiday);
    }

    #[test]
    #[test_log::test]
    fn test_fold_activities() {
        let work_day = Activity {
            id: Uuid::nil(),
            created_at: UtcDateTime::MIN,
            modified_at: UtcDateTime::MIN,
            name: Some("Working at the office".into()),
            class: ActivityClass::Work,
            time: Interval {
                start: Time::from_hms(9, 0, 0).unwrap(),
                end: Some(Time::from_hms(18, 0, 0).unwrap()),
            },
            description: None,
            tags: vec![],
            projects: vec![],
        };
        let break_time = Activity {
            id: Uuid::nil(),
            created_at: UtcDateTime::MIN,
            modified_at: UtcDateTime::MIN,
            name: Some("Lunch break".into()),
            class: ActivityClass::Break,
            time: Interval {
                start: Time::from_hms(12, 0, 0).unwrap(),
                end: Some(Time::from_hms(13, 0, 0).unwrap()),
            },
            description: None,
            tags: vec![],
            projects: vec![],
        };
        let project_meeting = Activity {
            id: Uuid::nil(),
            created_at: UtcDateTime::MIN,
            modified_at: UtcDateTime::MIN,
            name: Some("Project meeting".into()),
            class: ActivityClass::Work,
            time: Interval {
                start: Time::from_hms(10, 0, 0).unwrap(),
                end: Some(Time::from_hms(11, 0, 0).unwrap()),
            },
            description: None,
            tags: vec![],
            projects: vec![],
        };

        let project_meeting2 = Activity {
            id: Uuid::nil(),
            created_at: UtcDateTime::MIN,
            modified_at: UtcDateTime::MIN,
            name: Some("Project meeting 2".into()),
            class: ActivityClass::Work,
            time: Interval {
                start: Time::from_hms(10, 30, 0).unwrap(),
                end: Some(Time::from_hms(11, 30, 0).unwrap()),
            },
            description: None,
            tags: vec![],
            projects: vec![],
        };

        let project_meeting3 = Activity {
            id: Uuid::nil(),
            created_at: UtcDateTime::MIN,
            modified_at: UtcDateTime::MIN,
            name: Some("Project meeting 3".into()),
            class: ActivityClass::Work,
            time: Interval {
                start: Time::from_hms(13, 0, 0).unwrap(),
                end: Some(Time::from_hms(14, 0, 0).unwrap()),
            },
            description: None,
            tags: vec![],
            projects: vec![],
        };

        let day = Day {
            date: time::Date::from_calendar_date(2025, time::Month::January, 1).unwrap(),
            work_quota: time::Duration::hours(8),
            blockers: vec![],
            activities: vec![
                work_day,
                break_time,
                project_meeting,
                project_meeting2,
                project_meeting3,
            ],
        };

        let closure = day.calculate_activity_closure();
        for activity in &closure {
            println!(" - {}", activity);
        }

        assert_eq!(closure.len(), 8);
        assert_eq!(
            format!("{}", closure[0]),
            "09:00:00 - 10:00:00 (Work): Working at the office"
        );
        assert_eq!(
            format!("{}", closure[1]),
            "10:00:00 - 10:30:00 (Work): Project meeting; Working at the office"
        );
        assert_eq!(
            format!("{}", closure[2]),
            "10:30:00 - 11:00:00 (Work): Project meeting; Project meeting 2; Working at the office"
        );
        assert_eq!(
            format!("{}", closure[3]),
            "11:00:00 - 11:30:00 (Work): Project meeting 2; Working at the office"
        );
        assert_eq!(
            format!("{}", closure[4]),
            "11:30:00 - 12:00:00 (Work): Working at the office"
        );
        assert_eq!(
            format!("{}", closure[5]),
            "12:00:00 - 13:00:00 (Break): Lunch break; Working at the office"
        );
        assert_eq!(
            format!("{}", closure[6]),
            "13:00:00 - 14:00:00 (Work): Project meeting 3; Working at the office"
        );
        assert_eq!(
            format!("{}", closure[7]),
            "14:00:00 - 18:00:00 (Work): Working at the office"
        );
    }
}
