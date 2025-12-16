use semver::VersionReq;
use serde::{Deserialize, Serialize};

use std::fmt;

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Ecosystem {
    Rust,
    Npm,
}

impl fmt::Display for Ecosystem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Ecosystem::Rust => write!(f, "crates.io"),
            Ecosystem::Npm => write!(f, "npm"),
        }
    }
}

#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub struct PackageRef {
    pub ecosystem: Ecosystem,
    pub name: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
pub struct Advisory {
    pub id: String,
    pub crate_name: String,
    #[serde(with = "serde_version_req")]
    pub vulnerable_versions: VersionReq,
    pub description: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub details: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub cache_status: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_fetched_at: Option<u64>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct DatabaseSchema {
    pub schema_version: u32,
    pub advisories: Vec<Advisory>,
}

mod serde_version_req {
    use semver::VersionReq;
    use serde::{de, Deserializer, Serializer};
    use std::fmt;

    pub fn serialize<S>(req: &VersionReq, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&req.to_string())
    }

    pub fn deserialize<'de, D>(deserializer: D) -> Result<VersionReq, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct Visitor;

        impl<'de> de::Visitor<'de> for Visitor {
            type Value = VersionReq;

            fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                formatter.write_str("a valid semver version requirement string")
            }

            fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
            where
                E: de::Error,
            {
                VersionReq::parse(value).map_err(E::custom)
            }
        }

        deserializer.deserialize_str(Visitor)
    }
}
