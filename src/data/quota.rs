use crate::data::identifier::Identifier;
use serde::{Deserialize, Serialize};
use time::Duration;

/// inner quota data structure, no id
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct QuotaInner {
    /// identifier of the class
    pub class: Identifier,
    /// duration of the quota
    #[serde(with = "crate::serde::pretty_duration")]
    pub duration: Duration,
    /// optional description
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub description: Option<String>,
}

/// quota data structure
#[derive(Deserialize, Serialize, Debug, Clone)]
pub struct Quota {
    /// unique id
    pub id: uuid::Uuid,
    /// data
    #[serde(flatten)]
    pub inner: QuotaInner,
}

// /// week quotas
// #[derive(Deserialize, Serialize, Debug, Clone)]
// pub struct WeekQuotas {
//     /// quotas for the week
//     #[serde(skip_serializing_if = "Vec::is_empty", default)]
//     pub quotas: HashMap<u8, Vec<Quota>>,
// }
