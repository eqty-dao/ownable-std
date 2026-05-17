use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
/// Identity metadata for the source Ownable of a cross-ownable event.
pub struct OwnableEventSource {
    pub id: String,
    pub owner: String,
    pub issuer: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// Contract-facing payload for a semantic event emitted by another Ownable.
pub struct OwnableEvent {
    pub source: OwnableEventSource,
    pub event_type: String,
    pub attributes: Value,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnableEventError {
    UnexpectedEventType {
        expected: &'static str,
        actual: String,
    },
}

impl Display for OwnableEventError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            OwnableEventError::UnexpectedEventType { expected, actual } => {
                write!(
                    f,
                    "unexpected ownable event type: expected '{expected}', got '{actual}'"
                )
            }
        }
    }
}

impl Error for OwnableEventError {}

pub fn require_ownable_event_type(
    event: &OwnableEvent,
    expected: &'static str,
) -> Result<(), OwnableEventError> {
    if event.event_type == expected {
        Ok(())
    } else {
        Err(OwnableEventError::UnexpectedEventType {
            expected,
            actual: event.event_type.clone(),
        })
    }
}

pub fn source_matches(event: &OwnableEvent, id: &str, owner: &str, issuer: &str) -> bool {
    event.source.id == id && event.source.owner == owner && event.source.issuer == issuer
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn ownable_event_cbor_round_trip_preserves_source_and_attributes() {
        let event = OwnableEvent {
            source: OwnableEventSource {
                id: "ownable-123".to_string(),
                owner: "owner-1".to_string(),
                issuer: "issuer-1".to_string(),
            },
            event_type: "consume".to_string(),
            attributes: json!({"consumerId": "abc", "amount": 1}),
        };

        let encoded = crate::abi::cbor_to_vec(&event).expect("serialize ownable event");
        let decoded: OwnableEvent =
            crate::abi::cbor_from_slice(&encoded).expect("deserialize ownable event");

        assert_eq!(decoded, event);
    }

    #[test]
    fn require_ownable_event_type_accepts_matching_type() {
        let event = OwnableEvent {
            source: OwnableEventSource {
                id: "ownable-123".to_string(),
                owner: "owner-1".to_string(),
                issuer: "issuer-1".to_string(),
            },
            event_type: "consume".to_string(),
            attributes: json!({}),
        };

        assert_eq!(require_ownable_event_type(&event, "consume"), Ok(()));
    }

    #[test]
    fn require_ownable_event_type_rejects_wrong_type() {
        let event = OwnableEvent {
            source: OwnableEventSource {
                id: "ownable-123".to_string(),
                owner: "owner-1".to_string(),
                issuer: "issuer-1".to_string(),
            },
            event_type: "redeem".to_string(),
            attributes: json!({}),
        };

        assert_eq!(
            require_ownable_event_type(&event, "consume"),
            Err(OwnableEventError::UnexpectedEventType {
                expected: "consume",
                actual: "redeem".to_string(),
            })
        );
    }

    #[test]
    fn source_matches_checks_all_identity_fields() {
        let event = OwnableEvent {
            source: OwnableEventSource {
                id: "ownable-123".to_string(),
                owner: "owner-1".to_string(),
                issuer: "issuer-1".to_string(),
            },
            event_type: "consume".to_string(),
            attributes: json!({}),
        };

        assert!(source_matches(&event, "ownable-123", "owner-1", "issuer-1"));
        assert!(!source_matches(&event, "ownable-123", "owner-2", "issuer-1"));
    }
}
