use alloy_sol_types::SolType;
use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::error::Error;
use std::fmt::{Display, Formatter};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "camelCase")]
/// Canonical contract-facing payload for public events routed through Anchor.
pub struct PublicEvent {
    pub source: String,
    pub event_type: String,
    pub data: Binary,
    pub block_number: u64,
    pub transaction_hash: Binary,
    pub transaction_index: u32,
    pub log_index: u32,
}

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
pub enum PublicEventError {
    UnexpectedEventType {
        expected: &'static str,
        actual: String,
    },
    AbiDecode(String),
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum OwnableEventError {
    UnexpectedEventType {
        expected: &'static str,
        actual: String,
    },
}

impl Display for PublicEventError {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            PublicEventError::UnexpectedEventType { expected, actual } => {
                write!(
                    f,
                    "unexpected public event type: expected '{expected}', got '{actual}'"
                )
            }
            PublicEventError::AbiDecode(message) => {
                write!(f, "failed to decode ABI payload: {message}")
            }
        }
    }
}

impl Error for PublicEventError {}

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

pub fn require_event_type(
    event: &PublicEvent,
    expected: &'static str,
) -> Result<(), PublicEventError> {
    if event.event_type == expected {
        Ok(())
    } else {
        Err(PublicEventError::UnexpectedEventType {
            expected,
            actual: event.event_type.clone(),
        })
    }
}

pub fn decode_abi<T: SolType>(data: &[u8]) -> Result<T::RustType, PublicEventError> {
    T::abi_decode(data, true).map_err(|err| PublicEventError::AbiDecode(err.to_string()))
}

pub fn decode_abi_for<T: SolType>(
    event: &PublicEvent,
    expected_event_type: &'static str,
) -> Result<T::RustType, PublicEventError> {
    require_event_type(event, expected_event_type)?;
    decode_abi::<T>(event.data.as_slice())
}

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
    use alloy_sol_types::{sol, SolValue};
    use serde_json::json;

    #[test]
    fn public_event_cbor_round_trip_preserves_binary_fields() {
        let event = PublicEvent {
            source: "0xabc".to_string(),
            event_type: "consume".to_string(),
            data: Binary::from(vec![0xde, 0xad, 0xbe, 0xef]),
            block_number: 42,
            transaction_hash: Binary::from(vec![0xaa, 0xbb, 0xcc]),
            transaction_index: 2,
            log_index: 7,
        };

        let encoded = crate::abi::cbor_to_vec(&event).expect("serialize public event");
        let decoded: PublicEvent =
            crate::abi::cbor_from_slice(&encoded).expect("deserialize public event");

        assert_eq!(decoded, event);
    }

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
    fn require_event_type_accepts_matching_type() {
        let event = PublicEvent {
            source: "0xabc".to_string(),
            event_type: "consume".to_string(),
            data: Binary::default(),
            block_number: 0,
            transaction_hash: Binary::default(),
            transaction_index: 0,
            log_index: 0,
        };

        assert_eq!(require_event_type(&event, "consume"), Ok(()));
    }

    #[test]
    fn require_event_type_rejects_wrong_type() {
        let event = PublicEvent {
            source: "0xabc".to_string(),
            event_type: "redeem".to_string(),
            data: Binary::default(),
            block_number: 0,
            transaction_hash: Binary::default(),
            transaction_index: 0,
            log_index: 0,
        };

        assert_eq!(
            require_event_type(&event, "consume"),
            Err(PublicEventError::UnexpectedEventType {
                expected: "consume",
                actual: "redeem".to_string(),
            })
        );
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

    #[test]
    fn decode_abi_decodes_typed_payload() {
        type ConsumeEvent = sol!((uint32,bool));

        let expected = (123u32, true);
        let encoded = expected.abi_encode();

        let decoded: <ConsumeEvent as SolType>::RustType =
            decode_abi::<ConsumeEvent>(&encoded).expect("decode payload");

        assert_eq!(decoded, expected);
    }

    #[test]
    fn decode_abi_for_checks_event_type_before_decoding() {
        type ConsumeEvent = sol!((uint32,bool));

        let event = PublicEvent {
            source: "0xabc".to_string(),
            event_type: "redeem".to_string(),
            data: Binary::from(vec![0x00, 0x01]),
            block_number: 0,
            transaction_hash: Binary::default(),
            transaction_index: 0,
            log_index: 0,
        };

        let err =
            decode_abi_for::<ConsumeEvent>(&event, "consume").expect_err("must reject wrong event type");
        assert_eq!(
            err,
            PublicEventError::UnexpectedEventType {
                expected: "consume",
                actual: "redeem".to_string(),
            }
        );
    }

    #[test]
    fn decode_abi_rejects_invalid_bytes() {
        type ConsumeEvent = sol!((uint32,bool));

        let err = decode_abi::<ConsumeEvent>(&[0x01, 0x02]).expect_err("must reject invalid ABI");
        assert!(matches!(err, PublicEventError::AbiDecode(_)));
    }
}
