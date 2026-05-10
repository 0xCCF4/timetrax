use crate::data::identifier::Identifier;
use serde::{Deserialize, Serialize};
use time::{Duration, Weekday};

/// Expected duration for one activity class on a given day
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct ClassQuota {
    pub class: Identifier,
    #[serde(with = "crate::serde::pretty_duration")]
    pub duration: Duration,
}

/// Per-weekday quotas stored in the job config
#[derive(Deserialize, Serialize, Debug, Clone, Default)]
pub struct WeekQuotas {
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub monday: Vec<ClassQuota>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub tuesday: Vec<ClassQuota>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub wednesday: Vec<ClassQuota>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub thursday: Vec<ClassQuota>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub friday: Vec<ClassQuota>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub saturday: Vec<ClassQuota>,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub sunday: Vec<ClassQuota>,
}

impl WeekQuotas {
    #[must_use]
    pub fn for_weekday(&self, wd: Weekday) -> &Vec<ClassQuota> {
        match wd {
            Weekday::Monday => &self.monday,
            Weekday::Tuesday => &self.tuesday,
            Weekday::Wednesday => &self.wednesday,
            Weekday::Thursday => &self.thursday,
            Weekday::Friday => &self.friday,
            Weekday::Saturday => &self.saturday,
            Weekday::Sunday => &self.sunday,
        }
    }

    pub fn for_weekday_mut(&mut self, wd: Weekday) -> &mut Vec<ClassQuota> {
        match wd {
            Weekday::Monday => &mut self.monday,
            Weekday::Tuesday => &mut self.tuesday,
            Weekday::Wednesday => &mut self.wednesday,
            Weekday::Thursday => &mut self.thursday,
            Weekday::Friday => &mut self.friday,
            Weekday::Saturday => &mut self.saturday,
            Weekday::Sunday => &mut self.sunday,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::data::identifier::Identifier;

    fn make_quota(name: &str, hours: i64) -> ClassQuota {
        ClassQuota {
            class: Identifier::ByName(name.into()),
            duration: Duration::hours(hours),
        }
    }

    #[test]
    fn default_all_empty() {
        let wq = WeekQuotas::default();
        for wd in [
            Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday,
            Weekday::Thursday, Weekday::Friday, Weekday::Saturday, Weekday::Sunday,
        ] {
            assert!(wq.for_weekday(wd).is_empty(), "{wd:?} should be empty");
        }
    }

    #[test]
    fn for_weekday_routes_to_correct_field() {
        let mut wq = WeekQuotas::default();
        wq.monday.push(make_quota("work", 8));
        wq.saturday.push(make_quota("break", 1));

        assert_eq!(wq.for_weekday(Weekday::Monday).len(), 1);
        assert_eq!(wq.for_weekday(Weekday::Tuesday).len(), 0);
        assert_eq!(wq.for_weekday(Weekday::Saturday).len(), 1);
        assert_eq!(wq.for_weekday(Weekday::Sunday).len(), 0);
    }

    #[test]
    fn for_weekday_mut_pushes_to_correct_field() {
        let mut wq = WeekQuotas::default();
        wq.for_weekday_mut(Weekday::Wednesday).push(make_quota("work", 8));
        wq.for_weekday_mut(Weekday::Friday).push(make_quota("work", 6));

        assert_eq!(wq.wednesday.len(), 1);
        assert_eq!(wq.friday.len(), 1);
        assert_eq!(wq.monday.len(), 0);
    }

    #[test]
    fn for_weekday_mut_allows_upsert() {
        let mut wq = WeekQuotas::default();
        wq.for_weekday_mut(Weekday::Thursday).push(make_quota("work", 8));
        // Update
        wq.for_weekday_mut(Weekday::Thursday)[0].duration = Duration::hours(4);

        assert_eq!(wq.for_weekday(Weekday::Thursday)[0].duration, Duration::hours(4));
    }

    #[test]
    fn all_weekday_variants_reachable() {
        let mut wq = WeekQuotas::default();
        for wd in [
            Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday,
            Weekday::Thursday, Weekday::Friday, Weekday::Saturday, Weekday::Sunday,
        ] {
            wq.for_weekday_mut(wd).push(make_quota("work", 8));
        }
        for wd in [
            Weekday::Monday, Weekday::Tuesday, Weekday::Wednesday,
            Weekday::Thursday, Weekday::Friday, Weekday::Saturday, Weekday::Sunday,
        ] {
            assert_eq!(wq.for_weekday(wd).len(), 1);
        }
    }
}
