use crate::az_hash::AZHash;
use crate::data::BASIC_TIME_FORMAT;
use crate::data::day::ActivityClass;
use crate::data::interval::Interval;
use log::error;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use uuid::Uuid;

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

impl Display for Activity {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{} - {} ({:?}): {}",
            self.time
                .start
                .format(&*BASIC_TIME_FORMAT)
                .unwrap_or_else(|e| {
                    error!("Unable to format time: {e}. Report this as an issue.");
                    "<INVALID>".to_string()
                }),
            self.time
                .end
                .map(|t| t.format(&*BASIC_TIME_FORMAT).unwrap_or_else(|e| {
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

impl AZHash for Activity {
    fn az_hash(&self) -> String {
        self.id.az_hash()
    }
}
