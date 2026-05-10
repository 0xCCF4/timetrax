use crate::az_hash::AZHash;
use crate::data::BASIC_TIME_FORMAT;
use crate::data::activity::Activity;
use crate::data::job_config::JobConfig;
use serde::Serialize;
use serde_json::{Value, json};
use time::{Date, Time};

pub fn print_json(v: &impl Serialize) {
    match serde_json::to_string_pretty(v) {
        Ok(s) => println!("{s}"),
        Err(e) => eprintln!("JSON serialization error: {e}"),
    }
}

#[must_use]
pub fn fmt_time_json(t: Time) -> String {
    t.format(&*BASIC_TIME_FORMAT).unwrap_or_else(|_| "??:??:??".into())
}

/// Serialize an activity to a JSON value, including its computed hash.
/// `unique_prefix_len` - pass the min unique prefix length from `unique_prefix_map`,
/// or `None` to omit the `unique_prefix` field (e.g. for single-activity responses).
pub fn activity_json(
    activity: &Activity,
    date: Date,
    job_config: &JobConfig,
    unique_prefix_len: Option<usize>,
) -> Value {
    let hash = activity.az_hash_sha512();
    let class_name = job_config
        .resolve_class(&activity.class).map_or_else(|| activity.class.to_string(), |c| c.inner.name.clone());
    let projects: Vec<String> = activity
        .projects
        .iter()
        .map(|p| {
            job_config
                .resolve_project(p).map_or_else(|| p.to_string(), |pr| pr.inner.name.clone())
        })
        .collect();

    let mut v = json!({
        "date": date.to_string(),
        "id": activity.id.to_string(),
        "hash": hash,
        "class": class_name,
        "name": activity.name,
        "start": fmt_time_json(activity.time.start),
        "end": activity.time.end.map(fmt_time_json),
        "duration_seconds": activity.time.duration().map(time::Duration::whole_seconds),
        "projects": projects,
    });

    if let Some(ulen) = unique_prefix_len {
        v["unique_prefix"] = json!(&hash[..ulen.min(hash.len())]);
    }

    v
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::activity::Activity;
    use crate::data::identifier::Identifier;
    use crate::data::interval::Interval;
    use crate::data::job_config::JobConfig;
    use time::{Date, Time};
    use uuid::Uuid;

    fn make_activity(start: Time, end: Option<Time>) -> Activity {
        Activity {
            id: Uuid::nil(),
            name: Some("test".into()),
            class: Identifier::ByName("work".into()),
            time: Interval { start, end },
            projects: vec![],
        }
    }

    fn date() -> Date {
        Date::from_calendar_date(2024, time::Month::January, 15).unwrap()
    }

    #[test]
    fn fmt_time_json_formats_correctly() {
        let t = Time::from_hms(9, 5, 3).unwrap();
        assert_eq!(fmt_time_json(t), "09:05:03");
    }

    #[test]
    fn activity_json_open_interval_has_null_end_and_duration() {
        let jc = JobConfig::default();
        let a = make_activity(Time::from_hms(9, 0, 0).unwrap(), None);
        let v = activity_json(&a, date(), &jc, None);
        assert!(v["end"].is_null());
        assert!(v["duration_seconds"].is_null());
    }

    #[test]
    fn activity_json_closed_interval_has_duration() {
        let jc = JobConfig::default();
        let a = make_activity(
            Time::from_hms(9, 0, 0).unwrap(),
            Some(Time::from_hms(17, 0, 0).unwrap()),
        );
        let v = activity_json(&a, date(), &jc, None);
        assert_eq!(v["duration_seconds"], 8 * 3600);
        assert_eq!(v["end"], "17:00:00");
    }

    #[test]
    fn activity_json_includes_unique_prefix_when_provided() {
        let jc = JobConfig::default();
        let a = make_activity(Time::from_hms(9, 0, 0).unwrap(), None);
        let v = activity_json(&a, date(), &jc, Some(4));
        let hash = v["hash"].as_str().unwrap();
        let prefix = v["unique_prefix"].as_str().unwrap();
        assert_eq!(prefix, &hash[..4]);
    }

    #[test]
    fn activity_json_omits_unique_prefix_when_none() {
        let jc = JobConfig::default();
        let a = make_activity(Time::from_hms(9, 0, 0).unwrap(), None);
        let v = activity_json(&a, date(), &jc, None);
        assert!(v.get("unique_prefix").is_none());
    }

    #[test]
    fn activity_json_resolves_class_name() {
        let jc = JobConfig::default();
        let a = make_activity(Time::from_hms(9, 0, 0).unwrap(), None);
        let v = activity_json(&a, date(), &jc, None);
        assert_eq!(v["class"], "work");
    }
}
