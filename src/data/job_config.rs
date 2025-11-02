use crate::data::activity_class::{ActivityClass, ActivityClassInner};
use crate::data::identifier::Identifier;
use log::error;
use serde::{Deserialize, Serialize};
use std::borrow::Borrow;
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

#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct JobConfig {
    pub classes: Vec<ActivityClass>,
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
}
