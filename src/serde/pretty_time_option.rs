use crate::serde::pretty_time;
use serde::Deserialize;
use time::Time;

/// Serialize an `Option<Time>` as `"HH:MM:SS"` or JSON `null`.
///
/// # Errors
/// Returns a serialization error if the inner `Time` cannot be formatted.
pub fn serialize<S>(time: &Option<Time>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    match time {
        Some(time) => pretty_time::serialize(time, serializer),
        None => serializer.serialize_none(),
    }
}

/// Deserialize an `Option<Time>` from `"HH:MM:SS"` or JSON `null`.
///
/// # Errors
/// Returns a deserialization error if the string is present but not a valid time.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Option<Time>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let opt = Option::<String>::deserialize(deserializer)?;

    match opt {
        Some(s) => {
            let time = pretty_time::deserialize(serde::de::IntoDeserializer::into_deserializer(s))?;
            Ok(Some(time))
        }
        None => Ok(None),
    }
}
