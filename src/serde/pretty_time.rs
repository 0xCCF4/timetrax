use serde::Deserialize;
use std::sync::LazyLock;
use time::format_description::BorrowedFormatItem;
use time::{Time, format_description};

static TIME_FORMAT: LazyLock<Vec<BorrowedFormatItem>> = LazyLock::new(|| {
    format_description::parse("[hour]:[minute]:[second]").expect("Failed to parse time format")
});

/// Serialize a `Time` as `"HH:MM:SS"`.
///
/// # Errors
/// Returns a serialization error if formatting fails.
pub fn serialize<S>(time: &Time, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    time.format(&*TIME_FORMAT)
        .map_err(|e| serde::ser::Error::custom(format!("Failed to format time: {e}")))
        .and_then(|s| serializer.serialize_str(&s))
}

/// Deserialize a `Time` from `"HH:MM:SS"`.
///
/// # Errors
/// Returns a deserialization error if the string cannot be parsed as a time.
pub fn deserialize<'de, D>(deserializer: D) -> Result<Time, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;

    Time::parse(&s, &*TIME_FORMAT)
        .map_err(|e| serde::de::Error::custom(format!("Failed to parse time: {e}")))
}

#[cfg(test)]
mod tests {
    use serde::{Deserialize, Serialize};
    use time::Time;

    #[derive(Serialize, Deserialize, PartialEq, Debug)]
    struct Wrapper {
        #[serde(with = "super")]
        time: Time,
    }

    fn round(t: Time) -> Time {
        let json = serde_json::to_string(&Wrapper { time: t }).unwrap();
        serde_json::from_str::<Wrapper>(&json).unwrap().time
    }

    #[test]
    fn roundtrip_midnight() {
        assert_eq!(round(Time::MIDNIGHT), Time::MIDNIGHT);
    }

    #[test]
    fn roundtrip_noon() {
        let t = Time::from_hms(12, 0, 0).unwrap();
        assert_eq!(round(t), t);
    }

    #[test]
    fn roundtrip_with_seconds() {
        let t = Time::from_hms(14, 30, 45).unwrap();
        assert_eq!(round(t), t);
    }

    #[test]
    fn serialize_format_hh_mm_ss() {
        let w = Wrapper { time: Time::from_hms(9, 5, 3).unwrap() };
        let json = serde_json::to_string(&w).unwrap();
        assert_eq!(json, r#"{"time":"09:05:03"}"#);
    }

    #[test]
    fn deserialize_invalid_errors() {
        let json = r#"{"time":"not-a-time"}"#;
        assert!(serde_json::from_str::<Wrapper>(json).is_err());
    }
}
