use std::{error::Error, fmt};

use serde::{Serialize, de::DeserializeOwned};

/// Versioned serialized representation of an already-typed Rust payload.
///
/// This type preserves the kind/version/body needed for durable audit and replay tooling. Encoding
/// a value is not itself proof of authorization or semantic validation; callers must only persist
/// records that have crossed the application/core command or event validation boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SerializedRecord {
    kind: String,
    schema_version: u32,
    json: String,
}

impl SerializedRecord {
    pub fn encode<T: Serialize + ?Sized>(
        kind: impl Into<String>,
        schema_version: u32,
        value: &T,
    ) -> Result<Self, RecordCodecError> {
        let kind = kind.into();
        if kind.trim().is_empty() {
            return Err(RecordCodecError::EmptyKind);
        }
        if schema_version == 0 {
            return Err(RecordCodecError::ZeroSchemaVersion);
        }

        Ok(Self {
            kind,
            schema_version,
            json: serde_json::to_string(value).map_err(RecordCodecError::Json)?,
        })
    }

    #[must_use]
    pub fn kind(&self) -> &str {
        &self.kind
    }

    #[must_use]
    pub const fn schema_version(&self) -> u32 {
        self.schema_version
    }

    #[must_use]
    pub fn json(&self) -> &str {
        &self.json
    }

    pub fn decode<T: DeserializeOwned>(&self) -> Result<T, RecordCodecError> {
        serde_json::from_str(&self.json).map_err(RecordCodecError::Json)
    }
}

#[derive(Debug)]
pub enum RecordCodecError {
    EmptyKind,
    ZeroSchemaVersion,
    Json(serde_json::Error),
}

impl fmt::Display for RecordCodecError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyKind => formatter.write_str("record kind must not be empty"),
            Self::ZeroSchemaVersion => {
                formatter.write_str("record schema version must be positive")
            }
            Self::Json(error) => write!(formatter, "record JSON codec failed: {error}"),
        }
    }
}

impl Error for RecordCodecError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Json(error) => Some(error),
            Self::EmptyKind | Self::ZeroSchemaVersion => None,
        }
    }
}
