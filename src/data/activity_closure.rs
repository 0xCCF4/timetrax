use crate::data::activity::Activity;
use crate::data::interval::Interval;
use crate::data::job_config::JobConfig;
use itertools::Itertools;
use log::{error, trace};
use std::borrow::Borrow;
use std::collections::BinaryHeap;
use std::fmt::{Display, Formatter};
use time::Time;
use uuid::Uuid;

impl Activity {
    /// Folds multiple activities into a single activity
    /// the largest start time and smallest end time is used
    /// the highest priority class is used
    /// all other attributes are combined
    ///
    /// will return none if no activities are provided or the collapsed start_time > end_time
    pub fn fold_inner<Q: Borrow<Activity>, I: Iterator<Item = Q>>(
        job_config: &JobConfig,
        activities: I,
        start_time_limit: Option<&Time>,
        end_time_limit: Option<&Time>,
    ) -> Option<Activity> {
        let mut start_time = None;
        let mut end_time = None;
        let mut class = job_config.lowest_priority_class();
        let mut names = Vec::new();
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

            let activity_class = job_config.resolve_class(&activity.class).unwrap_or_else(|| {
                error!("Class {} not resolved. Did you removed it from the job config? Encountered on activity with ID {}", activity.class, activity.id);
                job_config.lowest_priority_class()
            });
            if activity_class.inner.priority > class.inner.priority {
                class = activity_class;
            }

            if let Some(activity_name) = &activity.name {
                if !names.contains(activity_name) {
                    names.push(activity_name.clone());
                }
            }

            for project in &activity.projects {
                projects.push(project.clone());
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

            names.sort();
            projects.sort();

            Some(Activity {
                id: Uuid::new_v4(),
                name: if names.len() == 0 {
                    None
                } else {
                    Some(names.into_iter().join("; ").into())
                },
                class: class.id.into(),
                time: Interval {
                    start: start_time,
                    end: end_time,
                },
                projects,
            })
        } else {
            None
        }
    }

    /// calculate activity closure for the day
    /// activity closure meaning a linear timeline of non-overlapping activities
    pub fn calculate_activity_closure<Q: Borrow<Activity>>(
        job_config: &JobConfig,
        activities: &Vec<Q>,
        start: Option<Time>,
        end: Option<Time>,
    ) -> Vec<Activity> {
        #[repr(transparent)]
        #[derive(Debug)]
        struct ActivitySortByEndTime<'a>(&'a Activity);

        impl Display for ActivitySortByEndTime<'_> {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                self.0.fmt(f)
            }
        }
        impl Ord for ActivitySortByEndTime<'_> {
            fn cmp(&self, other: &Self) -> std::cmp::Ordering {
                other
                    .0
                    .time
                    .end_time_or_end_of_day()
                    .cmp(&self.0.time.end_time_or_end_of_day())
            }
        }
        impl PartialOrd for ActivitySortByEndTime<'_> {
            fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
                Some(self.cmp(other))
            }
        }
        impl PartialEq for ActivitySortByEndTime<'_> {
            fn eq(&self, other: &Self) -> bool {
                self.cmp(other) == std::cmp::Ordering::Equal
            }
        }
        impl Eq for ActivitySortByEndTime<'_> {}
        fn fold_report(
            job_config: &JobConfig,
            stack: &BinaryHeap<ActivitySortByEndTime>,
            open_ended: &Vec<&Activity>,
            start_time: Option<&Time>,
            end_time: Option<&Time>,
        ) -> Option<Activity> {
            let folded = Activity::fold_inner(
                job_config,
                stack
                    .iter()
                    .map(|x| x.0)
                    .chain(open_ended.iter().map(|x| *x)),
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

        if let (Some(start), Some(end)) = (start, end) {
            if start >= end {
                return Vec::new();
            }
        }

        let mut activities: Vec<&Activity> = activities.iter().map(|x| x.borrow()).collect_vec();
        // sort by start time
        activities.sort_by(|a, b| a.time.start.cmp(&b.time.start));

        let mut closure: Vec<Activity> = Vec::with_capacity(activities.len() * 2);
        let mut activity_stack: BinaryHeap<ActivitySortByEndTime> =
            BinaryHeap::with_capacity(activities.len());
        let mut open_ended_activities = Vec::with_capacity(activities.len());

        trace!("Folding activities:");
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
                        job_config,
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
                job_config,
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
                job_config,
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

        if open_ended_activities.len() > 0 {
            trace!("   -> Folding remaining open-ended activities");
            if let Some(folded) = fold_report(
                job_config,
                &activity_stack,
                &open_ended_activities,
                last_activity_end.as_ref(),
                None,
            ) {
                closure.push(folded);
            }
        }

        if start.is_none() && end.is_none() {
            return closure;
        }

        trace!("Clamping closure to provided time limits");
        let mut result = Vec::with_capacity(closure.len());

        for mut activity in closure.into_iter() {
            trace!(" --> Processing activity: {}", activity);
            if let Some(start) = start {
                if activity.time.end_time_or_end_of_day() < start {
                    continue;
                }

                if activity.time.start < start && activity.time.end_time_or_end_of_day() > start {
                    activity.time.start = start;
                }
            }
            if let Some(end) = end {
                if activity.time.start >= end {
                    continue;
                }

                if activity.time.end_time_or_end_of_day() > end && activity.time.start < end {
                    activity.time.end = Some(end);
                }
            }

            result.push(activity);
        }

        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::activity_class::ActivityClass;
    use crate::data::identifier::Identifier;
    use time::{Time, UtcDateTime};

    #[test]
    fn test_fold_activities() {
        let job_config = JobConfig {
            classes: vec![
                ActivityClass {
                    id: Uuid::from_u128(1),
                    inner: crate::data::activity_class::ActivityClassInner {
                        name: "work".into(),
                        priority: 1,
                        description: None,
                    },
                },
                ActivityClass {
                    id: Uuid::from_u128(2),
                    inner: crate::data::activity_class::ActivityClassInner {
                        name: "break".into(),
                        priority: 2,
                        description: None,
                    },
                },
            ],
        };
        let work_day = Activity {
            id: Uuid::nil(),
            name: Some("Working at the office".into()),
            class: Identifier::ByName("work".into()),
            time: Interval {
                start: Time::from_hms(9, 0, 0).unwrap(),
                end: Some(Time::from_hms(18, 0, 0).unwrap()),
            },
            description: None,
            projects: vec![],
        };
        let break_time = Activity {
            id: Uuid::nil(),
            name: Some("Lunch break".into()),
            class: Identifier::ByName("break".into()),
            time: Interval {
                start: Time::from_hms(12, 0, 0).unwrap(),
                end: Some(Time::from_hms(13, 0, 0).unwrap()),
            },
            description: None,
            projects: vec![],
        };
        let project_meeting = Activity {
            id: Uuid::nil(),
            name: Some("Project meeting".into()),
            class: Identifier::ByName("work".into()),
            time: Interval {
                start: Time::from_hms(10, 0, 0).unwrap(),
                end: Some(Time::from_hms(11, 0, 0).unwrap()),
            },
            description: None,
            projects: vec![],
        };

        let project_meeting2 = Activity {
            id: Uuid::nil(),
            name: Some("Project meeting 2".into()),
            class: Identifier::ByName("work".into()),
            time: Interval {
                start: Time::from_hms(10, 30, 0).unwrap(),
                end: Some(Time::from_hms(11, 30, 0).unwrap()),
            },
            description: None,
            projects: vec![],
        };

        let project_meeting3 = Activity {
            id: Uuid::nil(),
            name: Some("Project meeting 3".into()),
            class: Identifier::ByName("work".into()),
            time: Interval {
                start: Time::from_hms(13, 0, 0).unwrap(),
                end: Some(Time::from_hms(14, 0, 0).unwrap()),
            },
            description: None,
            projects: vec![],
        };

        let day = vec![
            work_day,
            break_time,
            project_meeting,
            project_meeting2,
            project_meeting3,
        ];

        let closure = Activity::calculate_activity_closure(&job_config, &day);
        for activity in &closure {
            println!(" - {}", activity);
        }

        assert_eq!(closure.len(), 8);
        assert_eq!(
            format!("{}", closure[0]),
            "09:00:00 - 10:00:00: Working at the office"
        );
        assert_eq!(closure[0].class, Uuid::from_u128(1).into());
        assert_eq!(
            format!("{}", closure[1]),
            "10:00:00 - 10:30:00: Project meeting; Working at the office"
        );
        assert_eq!(closure[1].class, Uuid::from_u128(1).into());
        assert_eq!(
            format!("{}", closure[2]),
            "10:30:00 - 11:00:00: Project meeting; Project meeting 2; Working at the office"
        );
        assert_eq!(closure[2].class, Uuid::from_u128(1).into());
        assert_eq!(
            format!("{}", closure[3]),
            "11:00:00 - 11:30:00: Project meeting 2; Working at the office"
        );
        assert_eq!(closure[3].class, Uuid::from_u128(1).into());
        assert_eq!(
            format!("{}", closure[4]),
            "11:30:00 - 12:00:00: Working at the office"
        );
        assert_eq!(closure[4].class, Uuid::from_u128(1).into());
        assert_eq!(
            format!("{}", closure[5]),
            "12:00:00 - 13:00:00: Lunch break; Working at the office"
        );
        assert_eq!(closure[5].class, Uuid::from_u128(2).into());
        assert_eq!(
            format!("{}", closure[6]),
            "13:00:00 - 14:00:00: Project meeting 3; Working at the office"
        );
        assert_eq!(closure[6].class, Uuid::from_u128(1).into());
        assert_eq!(
            format!("{}", closure[7]),
            "14:00:00 - 18:00:00: Working at the office"
        );
        assert_eq!(closure[7].class, Uuid::from_u128(1).into());
    }
}
