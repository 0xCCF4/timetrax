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
        Identifier::ByName(value.replace("@", ""))
    }
}

impl Display for IdentifierConvertError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            IdentifierConvertError::Empty => write!(f, "Identifier string is empty"),
            IdentifierConvertError::UuidFormat(e) => write!(f, "UUID format error: {}", e),
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
            return Err(IdentifierConvertError::Empty);
        } else if s.starts_with("@") {
            Ok(Identifier::ByName(s[1..].to_string()))
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
            Identifier::Uuid(id) => write!(f, "{}", id),
            Identifier::ByName(name) => write!(f, "@{}", name),
        }
    }
}
