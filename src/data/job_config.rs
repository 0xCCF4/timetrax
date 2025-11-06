use crate::data::activity_class::{ActivityClass, ActivityClassInner};
use crate::data::identifier::Identifier;
use crate::data::project::Project;
use log::error;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
use std::str::FromStr;
use std::sync::LazyLock;
use uuid::Uuid;

static DUMMY_ACTIVITY_CLASS: LazyLock<ActivityClass> = LazyLock::new(|| ActivityClass {
    id: Uuid::nil(),
    inner: ActivityClassInner {
        priority: 0,
        name: "<UNDEFINED>".to_string(),
        description: Some("No classes specified in job config. Using a dummy class.".to_string()),
    },
});

/// configuration file for the job instance
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JobConfig {
    /// activity classes
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub classes: Vec<ActivityClass>,
    /// projects
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub projects: Vec<Project>,
    // /// daily quotas
}

impl JobConfig {
    pub fn lowest_priority_class(&self) -> &ActivityClass {
        self.classes.iter().min_by(|a, b| a.inner.priority.cmp(&b.inner.priority)).unwrap_or_else(|| {
            error!("Your job configuration does not specify any activity classes. This will lead to wrong total time calculation!");
            &DUMMY_ACTIVITY_CLASS
        })
    }

    pub fn resolve_class<Q: Borrow<Identifier>>(&self, identifier: Q) -> Option<&ActivityClass> {
        self.classes
            .iter()
            .find(|class| class.identifier_matches(identifier.borrow()))
    }

    pub fn resolve_class_mut<Q: Borrow<Identifier>>(
        &mut self,
        identifier: Q,
    ) -> Option<&mut ActivityClass> {
        self.classes
            .iter_mut()
            .find(|class| class.identifier_matches(identifier.borrow()))
    }

    pub fn resolve_project<Q: Borrow<Identifier>>(&self, identifier: Q) -> Option<&Project> {
        self.projects
            .iter()
            .find(|project| project.identifier_matches(identifier.borrow()))
    }

    pub fn resolve_project_mut<Q: Borrow<Identifier>>(
        &mut self,
        identifier: Q,
    ) -> Option<&mut Project> {
        self.projects
            .iter_mut()
            .find(|project| project.identifier_matches(identifier.borrow()))
    }
}

impl Default for JobConfig {
    fn default() -> Self {
        Self {
            classes: vec![
                ActivityClass {
                    id: Uuid::from_str("181e5c24-2a6d-49da-882b-60a07a38e2b0").unwrap(),
                    inner: ActivityClassInner {
                        priority: 0,
                        name: "work".to_string(),
                        description: Some("Work. Counted against work quota.".to_string()),
                    }
                },
                ActivityClass {
                    id: Uuid::from_str("a7c3da19-648f-43e3-abc1-874e49e79bde").unwrap(),
                    inner: ActivityClassInner {
                        priority: 5,
                        name: "break".to_string(),
                        description: Some("Activities classified as a short break during work. Legally required break-time.".to_string()),
                    }
                },
                ActivityClass {
                    id: Uuid::from_str("e45b8156-efc5-492f-b790-80d6be52b74f").unwrap(),
                    inner: ActivityClassInner {
                        priority: 10,
                        name: "holiday".to_string(),
                        description: Some("Holiday/Vacation time.".to_string()),
                    }
                }
            ],
            projects: vec![],
        }
    }
}
