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
    /// Fold multiple activities into a single activity.
    ///
    /// Uses the latest start time, earliest end time, highest-priority class, and merged projects.
    /// Returns `None` if no activities are provided or if collapsed `start_time > end_time`.
    #[must_use]
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
            if let Some(end_time) = end_time.as_mut()
                && let Some(activity_end) = activity.time.end
                    && activity_end < *end_time {
                        *end_time = activity_end;
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

            if let Some(activity_name) = &activity.name
                && !names.contains(activity_name) {
                    names.push(activity_name.clone());
                }

            for project in &activity.projects {
                projects.push(project.clone());
            }
        }

        if let Some(mut start_time) = start_time {
            if let Some(start_time_limit) = start_time_limit
                && start_time < *start_time_limit {
                    start_time = *start_time_limit;
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

            if let Some(end_time) = end_time
                && start_time > end_time {
                    return None;
                }

            names.sort();
            projects.sort();

            Some(Activity {
                id: Uuid::new_v4(),
                name: if names.is_empty() {
                    None
                } else {
                    Some(names.into_iter().join("; "))
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

    /// Calculate activity closure for the day: a linear timeline of non-overlapping activities.
    #[must_use]
    #[allow(clippy::too_many_lines)]
    pub fn calculate_activity_closure<Q: Borrow<Activity>>(
        job_config: &JobConfig,
        activities: &[Q],
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
                    .chain(open_ended.iter().copied()),
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

        if let (Some(start), Some(end)) = (start, end)
            && start >= end {
                return Vec::new();
            }

        let mut activities: Vec<&Activity> = activities.iter().map(Borrow::borrow).collect_vec();
        // sort by start time
        activities.sort_by_key(|a| a.time.start);

        let mut closure: Vec<Activity> = Vec::with_capacity(activities.len() * 2);
        let mut activity_stack: BinaryHeap<ActivitySortByEndTime> =
            BinaryHeap::with_capacity(activities.len());
        let mut open_ended_activities = Vec::with_capacity(activities.len());

        trace!("Folding activities:");
        if log::max_level() >= log::LevelFilter::Trace {
            trace!("  Input activities:");
            for activity in &activities {
                trace!("   - {activity}");
            }
        }

        let mut last_activity_end = None;
        for activity in activities {
            trace!(" - Processing activity: {activity}");

            while let Some(top_activity) = activity_stack.peek() {
                trace!("   - Current activity stack:");
                if log::max_level() >= log::LevelFilter::Trace {
                    for ac in &activity_stack {
                        trace!("     {ac}");
                    }
                }

                if top_activity.0.time.end_time_or_end_of_day() <= activity.time.start {
                    trace!("   -> Dropping activity from stack: {top_activity}");

                    if let Some(folded) = fold_report(
                        job_config,
                        &activity_stack,
                        &open_ended_activities,
                        last_activity_end.as_ref(),
                        Some(&top_activity.0.time.end_time_or_end_of_day()),
                    ) {
                        trace!(
                            "   -> Folding current stack up to end of dropped activity: {folded}"
                        );
                        closure.push(folded);
                    }

                    last_activity_end = Some(top_activity.0.time.end_time_or_end_of_day());

                    activity_stack.pop();
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
                    "   -> Folding current stack up to start of new activity: {folded}"
                );
                closure.push(folded);
            }

            if activity.time.is_complete() {
                trace!("   -> Pushing activity to stack: {activity}");
                activity_stack.push(ActivitySortByEndTime(activity));
            } else {
                trace!("   -> Adding open-ended activity: {activity}");
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
            trace!("   -> Dropping activity from stack: {activity}");

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

            activity_stack.pop();
        }

        if !open_ended_activities.is_empty() {
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

        for mut activity in closure {
            trace!(" --> Processing activity: {activity}");
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
    use crate::data::activity_class::{ActivityClass, ActivityClassInner};
    use crate::data::identifier::Identifier;
    use crate::data::week_quota::WeekQuotas;
    use time::Time;

    fn make_job_config() -> JobConfig {
        JobConfig {
            classes: vec![
                ActivityClass {
                    id: Uuid::from_u128(1),
                    inner: ActivityClassInner {
                        name: "work".into(),
                        priority: 1,
                        description: None,
                    },
                },
                ActivityClass {
                    id: Uuid::from_u128(2),
                    inner: ActivityClassInner {
                        name: "break".into(),
                        priority: 2,
                        description: None,
                    },
                },
            ],
            projects: vec![],
            week_quotas: WeekQuotas::default(),
        }
    }

    fn make_activity(
        name: &str,
        class: &str,
        start: (u8, u8, u8),
        end: Option<(u8, u8, u8)>,
    ) -> Activity {
        Activity {
            id: Uuid::new_v4(),
            name: Some(name.into()),
            class: Identifier::ByName(class.into()),
            time: Interval {
                start: Time::from_hms(start.0, start.1, start.2).unwrap(),
                end: end.map(|(h, m, s)| Time::from_hms(h, m, s).unwrap()),
            },
            projects: vec![],
        }
    }

    #[test]
    fn closure_empty_input_returns_empty() {
        let job_config = make_job_config();
        let day: Vec<Activity> = vec![];
        let closure = Activity::calculate_activity_closure(&job_config, &day, None, None);
        assert!(closure.is_empty());
    }

    #[test]
    fn closure_single_activity_passthrough() {
        let job_config = make_job_config();
        let day = vec![make_activity("work", "work", (9, 0, 0), Some((17, 0, 0)))];
        let closure = Activity::calculate_activity_closure(&job_config, &day, None, None);
        assert_eq!(closure.len(), 1);
        assert_eq!(format!("{}", closure[0]), "09:00:00 - 17:00:00: work");
        assert_eq!(closure[0].class, Uuid::from_u128(1).into());
    }

    #[test]
    fn closure_non_overlapping_activities_preserve_order() {
        let job_config = make_job_config();
        let day = vec![
            make_activity("morning", "work", (8, 0, 0), Some((12, 0, 0))),
            make_activity("lunch", "break", (12, 0, 0), Some((13, 0, 0))),
            make_activity("afternoon", "work", (13, 0, 0), Some((17, 0, 0))),
        ];
        let closure = Activity::calculate_activity_closure(&job_config, &day, None, None);
        assert_eq!(closure.len(), 3);
        assert_eq!(format!("{}", closure[0]), "08:00:00 - 12:00:00: morning");
        assert_eq!(format!("{}", closure[1]), "12:00:00 - 13:00:00: lunch");
        assert_eq!(format!("{}", closure[2]), "13:00:00 - 17:00:00: afternoon");
    }

    #[test]
    fn closure_break_interrupts_work() {
        let job_config = make_job_config();
        let day = vec![
            make_activity("working", "work", (9, 0, 0), Some((17, 0, 0))),
            make_activity("lunch", "break", (12, 0, 0), Some((13, 0, 0))),
        ];
        let closure = Activity::calculate_activity_closure(&job_config, &day, None, None);
        // Expect: 9-12 work, 12-13 break (break class wins), 13-17 work
        assert_eq!(closure.len(), 3);
        assert_eq!(closure[0].class, Uuid::from_u128(1).into()); // work
        assert_eq!(closure[1].class, Uuid::from_u128(2).into()); // break wins
        assert_eq!(closure[2].class, Uuid::from_u128(1).into()); // work
    }

    #[test]
    fn closure_time_limits_clamp_output() {
        let job_config = make_job_config();
        let day = vec![make_activity("all-day", "work", (6, 0, 0), Some((22, 0, 0)))];
        let start = Time::from_hms(9, 0, 0).unwrap();
        let end = Time::from_hms(17, 0, 0).unwrap();
        let closure =
            Activity::calculate_activity_closure(&job_config, &day, Some(start), Some(end));
        assert_eq!(closure.len(), 1);
        assert_eq!(
            format!("{}", closure[0]),
            "09:00:00 - 17:00:00: all-day"
        );
    }

    #[test]
    fn closure_complex_overlapping_day() {
        let job_config = make_job_config();
        let day = vec![
            make_activity("Working at the office", "work", (9, 0, 0), Some((18, 0, 0))),
            make_activity("Lunch break", "break", (12, 0, 0), Some((13, 0, 0))),
            make_activity("Project meeting", "work", (10, 0, 0), Some((11, 0, 0))),
            make_activity("Project meeting 2", "work", (10, 30, 0), Some((11, 30, 0))),
            make_activity("Project meeting 3", "work", (13, 0, 0), Some((14, 0, 0))),
        ];

        let closure = Activity::calculate_activity_closure(&job_config, &day, None, None);

        assert_eq!(closure.len(), 8);
        assert_eq!(format!("{}", closure[0]), "09:00:00 - 10:00:00: Working at the office");
        assert_eq!(closure[0].class, Uuid::from_u128(1).into());
        assert_eq!(format!("{}", closure[1]), "10:00:00 - 10:30:00: Project meeting; Working at the office");
        assert_eq!(closure[1].class, Uuid::from_u128(1).into());
        assert_eq!(format!("{}", closure[2]), "10:30:00 - 11:00:00: Project meeting; Project meeting 2; Working at the office");
        assert_eq!(closure[2].class, Uuid::from_u128(1).into());
        assert_eq!(format!("{}", closure[3]), "11:00:00 - 11:30:00: Project meeting 2; Working at the office");
        assert_eq!(closure[3].class, Uuid::from_u128(1).into());
        assert_eq!(format!("{}", closure[4]), "11:30:00 - 12:00:00: Working at the office");
        assert_eq!(closure[4].class, Uuid::from_u128(1).into());
        assert_eq!(format!("{}", closure[5]), "12:00:00 - 13:00:00: Lunch break; Working at the office");
        assert_eq!(closure[5].class, Uuid::from_u128(2).into());
        assert_eq!(format!("{}", closure[6]), "13:00:00 - 14:00:00: Project meeting 3; Working at the office");
        assert_eq!(closure[6].class, Uuid::from_u128(1).into());
        assert_eq!(format!("{}", closure[7]), "14:00:00 - 18:00:00: Working at the office");
        assert_eq!(closure[7].class, Uuid::from_u128(1).into());
    }
}
