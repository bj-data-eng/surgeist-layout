//! The versioned, layout-owned envelope around opaque variant measurements.

use std::error::Error;
use std::fmt;

use serde::{Deserialize, Deserializer};
use serde_json::value::RawValue;

pub(super) const VERSION: u32 = 1;

#[cfg(test)]
#[derive(Debug, Eq, PartialEq)]
pub(super) enum EnvelopeErrorKind {
    Decode,
    MissingVersion,
    UnsupportedVersion(u32),
    MissingVariant,
}

#[derive(Debug)]
pub(super) enum EnvelopeError {
    Decode {
        field_path: String,
        source: serde_json::Error,
    },
    MissingVersion,
    UnsupportedVersion(u32),
    MissingVariant(&'static str),
}

impl EnvelopeError {
    pub(super) fn field_path(&self) -> &str {
        match self {
            Self::Decode { field_path, .. } => field_path,
            Self::MissingVersion | Self::UnsupportedVersion(_) => "schemaVersion",
            Self::MissingVariant(field) => field,
        }
    }

    #[cfg(test)]
    pub(super) fn kind(&self) -> EnvelopeErrorKind {
        match self {
            Self::Decode { .. } => EnvelopeErrorKind::Decode,
            Self::MissingVersion => EnvelopeErrorKind::MissingVersion,
            Self::UnsupportedVersion(version) => EnvelopeErrorKind::UnsupportedVersion(*version),
            Self::MissingVariant(_) => EnvelopeErrorKind::MissingVariant,
        }
    }
}

impl fmt::Display for EnvelopeError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "measurement envelope {}: ", self.field_path())?;
        match self {
            Self::Decode { source, .. } => write!(formatter, "{source}"),
            Self::MissingVersion => write!(formatter, "schemaVersion is required"),
            Self::UnsupportedVersion(version) => write!(
                formatter,
                "schemaVersion {version} is unsupported; expected {VERSION}"
            ),
            Self::MissingVariant(_) => write!(formatter, "declared variant is required"),
        }
    }
}

impl Error for EnvelopeError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            Self::Decode { source, .. } => Some(source),
            _ => None,
        }
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
struct WireEnvelope {
    #[serde(default, deserialize_with = "present_raw")]
    schema_version: Option<Box<RawValue>>,
    #[serde(default, deserialize_with = "present_raw")]
    border_box_ltr_data: Option<Box<RawValue>>,
    #[serde(default, deserialize_with = "present_raw")]
    content_box_ltr_data: Option<Box<RawValue>>,
    #[serde(default, deserialize_with = "present_raw")]
    border_box_rtl_data: Option<Box<RawValue>>,
    #[serde(default, deserialize_with = "present_raw")]
    content_box_rtl_data: Option<Box<RawValue>>,
}

// A present null is retained for type validation, rather than treated as absent.
fn present_raw<'de, D: Deserializer<'de>>(
    deserializer: D,
) -> Result<Option<Box<RawValue>>, D::Error> {
    Box::<RawValue>::deserialize(deserializer).map(Some)
}

pub(super) fn decode(raw: &str) -> Result<[Box<RawValue>; 4], EnvelopeError> {
    let mut deserializer = serde_json::Deserializer::from_str(raw);
    let wire: WireEnvelope =
        serde_path_to_error::deserialize(&mut deserializer).map_err(|error| {
            let path = error.path().to_string();
            EnvelopeError::Decode {
                field_path: if path.is_empty() {
                    "$".to_string()
                } else {
                    path
                },
                source: error.into_inner(),
            }
        })?;
    deserializer.end().map_err(|source| EnvelopeError::Decode {
        field_path: "$".to_string(),
        source,
    })?;
    let version = wire.schema_version.ok_or(EnvelopeError::MissingVersion)?;
    let version: u32 =
        serde_json::from_str(version.get()).map_err(|source| EnvelopeError::Decode {
            field_path: "schemaVersion".to_string(),
            source,
        })?;
    if version != VERSION {
        return Err(EnvelopeError::UnsupportedVersion(version));
    }
    let required = |value: Option<Box<RawValue>>, field: &'static str| {
        value.ok_or(EnvelopeError::MissingVariant(field))
    };
    Ok([
        required(wire.border_box_ltr_data, "borderBoxLtrData")?,
        required(wire.content_box_ltr_data, "contentBoxLtrData")?,
        required(wire.border_box_rtl_data, "borderBoxRtlData")?,
        required(wire.content_box_rtl_data, "contentBoxRtlData")?,
    ])
}

#[cfg(test)]
mod tests {
    use super::*;

    fn payload(version: &str) -> String {
        format!(
            "{{{version}\"borderBoxLtrData\":{{}},\"contentBoxLtrData\":{{}},\"borderBoxRtlData\":{{}},\"contentBoxRtlData\":{{}}}}"
        )
    }

    #[test]
    fn missing_and_unsupported_versions_have_distinct_contextual_errors() {
        let missing = decode(&payload("")).unwrap_err();
        assert_eq!(missing.kind(), EnvelopeErrorKind::MissingVersion);
        assert_eq!(missing.field_path(), "schemaVersion");
        let unsupported = decode(&payload("\"schemaVersion\":2,")).unwrap_err();
        assert_eq!(unsupported.kind(), EnvelopeErrorKind::UnsupportedVersion(2));
        assert_eq!(unsupported.field_path(), "schemaVersion");
    }

    #[test]
    fn version_must_be_an_integer_and_decode_errors_retain_the_json_source() {
        for version in ["null", "1.0", "true", "\"1\"", "[]", "{}", "-1"] {
            let error = decode(&payload(&format!("\"schemaVersion\":{version},"))).unwrap_err();
            assert_eq!(error.kind(), EnvelopeErrorKind::Decode, "{version}");
            assert_eq!(error.field_path(), "schemaVersion", "{version}");
            assert!(error.source().is_some());
        }
    }

    #[test]
    fn unknown_duplicate_and_trailing_envelope_data_are_rejected() {
        for prefix in [
            "\"schemaVersion\":1,\"extra\":true,",
            "\"schemaVersion\":1,\"schemaVersion\":1,",
            "\"schemaVersion\":1,\"borderBoxLtrData\":{},",
        ] {
            assert_eq!(
                decode(&payload(prefix)).unwrap_err().kind(),
                EnvelopeErrorKind::Decode
            );
        }
        assert_eq!(
            decode(&(payload("\"schemaVersion\":1,") + " {}"))
                .unwrap_err()
                .kind(),
            EnvelopeErrorKind::Decode
        );
    }

    #[test]
    fn every_variant_is_required_and_raw_duplicate_node_fields_survive_ingress() {
        let error = decode("{\"schemaVersion\":1}").unwrap_err();
        assert_eq!(error.kind(), EnvelopeErrorKind::MissingVariant);
        assert_eq!(error.field_path(), "borderBoxLtrData");
        let payload = payload("\"schemaVersion\":1,").replacen(
            "\"borderBoxLtrData\":{}",
            "\"borderBoxLtrData\":{\"children\":[],\"children\":[]}",
            1,
        );
        let variants = decode(&payload).unwrap();
        assert_eq!(variants[0].get(), "{\"children\":[],\"children\":[]}");
    }
}
