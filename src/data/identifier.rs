use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter};
use std::str::FromStr;
use uuid::Uuid;

#[derive(Serialize, Debug, Clone, PartialEq, PartialOrd, Ord, Eq)]
#[serde(into = "String")]
pub enum Identifier {
    Uuid(Uuid),
    ByName(String),
}

#[derive(Debug)]
pub enum IdentifierConvertError {
    Empty,
    UuidFormat(uuid::Error),
}

impl From<Uuid> for Identifier {
    fn from(id: Uuid) -> Self {
        Identifier::Uuid(id)
    }
}

impl From<String> for Identifier {
    fn from(value: String) -> Self {
        Identifier::ByName(value.replace('@', ""))
    }
}

impl Display for IdentifierConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentifierConvertError::Empty => write!(f, "Identifier string is empty"),
            IdentifierConvertError::UuidFormat(e) => write!(f, "UUID format error: {e}"),
        }
    }
}

impl std::error::Error for IdentifierConvertError {}

impl<'de> Deserialize<'de> for Identifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        Identifier::from_str(&s).map_err(serde::de::Error::custom)
    }
}

impl FromStr for Identifier {
    type Err = IdentifierConvertError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.is_empty() {
            Err(IdentifierConvertError::Empty)
        } else if let Some(name) = s.strip_prefix('@') {
            Ok(Identifier::ByName(name.to_string()))
        } else {
            Uuid::from_str(s)
                .map(Identifier::Uuid)
                .map_err(IdentifierConvertError::UuidFormat)
        }
    }
}

impl From<&Identifier> for String {
    fn from(id: &Identifier) -> Self {
        match id {
            Identifier::Uuid(id) => id.to_string(),
            Identifier::ByName(name) => format!("@{name}"),
        }
    }
}

impl From<Identifier> for String {
    fn from(value: Identifier) -> Self {
        From::from(&value)
    }
}

impl Display for Identifier {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Identifier::Uuid(id) => write!(f, "{id}"),
            Identifier::ByName(name) => write!(f, "@{name}"),
        }
    }
}

impl Identifier {
    /// Raw string representation without the `@` prefix.
    #[must_use]
    pub fn as_str_repr(&self) -> &str {
        match self {
            Identifier::Uuid(_) => "<unknown-uuid>",
            Identifier::ByName(name) => name.as_str(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_uuid() {
        let id: Identifier = "181e5c24-2a6d-49da-882b-60a07a38e2b0".parse().unwrap();
        assert!(matches!(id, Identifier::Uuid(_)));
    }

    #[test]
    fn parse_name_with_at() {
        let id: Identifier = "@work".parse().unwrap();
        assert_eq!(id, Identifier::ByName("work".into()));
    }

    #[test]
    fn parse_name_strips_at() {
        let id: Identifier = "@my-project".parse().unwrap();
        assert!(matches!(id, Identifier::ByName(n) if n == "my-project"));
    }

    #[test]
    fn parse_empty_is_error() {
        let result = "".parse::<Identifier>();
        assert!(result.is_err());
    }

    #[test]
    fn parse_non_uuid_without_at_is_error() {
        let result = "not-a-uuid-and-no-at".parse::<Identifier>();
        assert!(result.is_err());
    }

    #[test]
    fn display_uuid() {
        let uuid = Uuid::from_u128(42);
        let id = Identifier::Uuid(uuid);
        assert_eq!(id.to_string(), uuid.to_string());
    }

    #[test]
    fn display_by_name_adds_at() {
        let id = Identifier::ByName("work".into());
        assert_eq!(id.to_string(), "@work");
    }

    #[test]
    fn as_str_repr_returns_name() {
        let id = Identifier::ByName("myproject".into());
        assert_eq!(id.as_str_repr(), "myproject");
    }

    #[test]
    fn as_str_repr_uuid_returns_placeholder() {
        let id = Identifier::Uuid(Uuid::new_v4());
        assert_eq!(id.as_str_repr(), "<unknown-uuid>");
    }

    #[test]
    fn from_string_keeps_name() {
        let id = Identifier::from("@hello".to_string());
        assert_eq!(id, Identifier::ByName("hello".into()));
    }

    #[test]
    fn roundtrip_via_string_name() {
        let original = Identifier::ByName("break".into());
        let s: String = original.clone().into();
        let parsed: Identifier = s.parse().unwrap();
        assert_eq!(parsed, original);
    }

    #[test]
    fn roundtrip_via_string_uuid() {
        let uuid = Uuid::new_v4();
        let original = Identifier::Uuid(uuid);
        let s: String = original.clone().into();
        let parsed: Identifier = s.parse().unwrap();
        assert_eq!(parsed, original);
    }
}
