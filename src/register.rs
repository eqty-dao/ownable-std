use alloy_sol_types::SolType;
use cosmwasm_std::Binary;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
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
#[serde(rename_all = "camelCase")]
/// Request payload for JS-driven public-event ABI encoding.
pub struct EncodePublicEventRequest {
    pub event_type: String,
    pub data: Binary,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum PublicEventError {
    UnexpectedEventType {
        expected: &'static str,
        actual: String,
    },
    AbiDecode(String),
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

pub fn encode_abi<T: SolType>(value: &T::RustType) -> Vec<u8> {
    T::abi_encode(value)
}

pub fn decode_abi_for<T: SolType>(
    event: &PublicEvent,
    expected_event_type: &'static str,
) -> Result<T::RustType, PublicEventError> {
    require_event_type(event, expected_event_type)?;
    decode_abi::<T>(event.data.as_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use alloy_sol_types::sol;

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
    fn encode_public_event_request_cbor_round_trip_preserves_fields() {
        let request = EncodePublicEventRequest {
            event_type: "consume".to_string(),
            data: Binary::from(vec![0xa1, 0x66, b'a', b'm', b'o', b'u', b'n', b't']),
        };

        let encoded = crate::abi::cbor_to_vec(&request).expect("serialize encode request");
        let decoded: EncodePublicEventRequest =
            crate::abi::cbor_from_slice(&encoded).expect("deserialize encode request");

        assert_eq!(decoded, request);
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
    fn decode_abi_decodes_typed_payload() {
        type ConsumeEvent = sol!((uint32,bool));

        let expected = (123u32, true);
        let encoded = encode_abi::<ConsumeEvent>(&expected);

        let decoded: <ConsumeEvent as SolType>::RustType =
            decode_abi::<ConsumeEvent>(&encoded).expect("decode payload");

        assert_eq!(decoded, expected);
    }

    #[test]
    fn encode_abi_round_trips_through_decode() {
        type ConsumeEvent = sol!((uint32,bool));

        let expected = (456u32, false);
        let encoded = encode_abi::<ConsumeEvent>(&expected);
        let decoded = decode_abi::<ConsumeEvent>(&encoded).expect("decode encoded payload");

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
