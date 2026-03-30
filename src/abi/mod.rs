use crate::IdbStateDump;
use cosmwasm_std::Response;
use serde::{Deserialize, Serialize};
use std::fmt::Display;
use std::panic::UnwindSafe;

pub const HOST_ABI_VERSION: &str = "1";
pub const HOST_ABI_MANIFEST_FIELD: &str = "ownablesAbi";
pub const HOST_ABI_WIRE_FORMAT: &str = "cbor";
pub const HOST_ABI_WIRE_FORMAT_MANIFEST_FIELD: &str = "wireFormat";
pub const ERR_INVALID_POINTER: &str = "INVALID_POINTER";
pub const ERR_INVALID_CBOR: &str = "INVALID_CBOR";
pub const ERR_SERIALIZATION_FAILED: &str = "SERIALIZATION_FAILED";
pub const ERR_HANDLER_PANIC: &str = "HANDLER_PANIC";

/// Error object returned by the host ABI envelope.
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HostAbiError {
    pub code: Option<String>,
    pub message: String,
}

impl HostAbiError {
    /// Creates an error without a machine-readable code.
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            code: None,
            message: message.into(),
        }
    }

    /// Creates an error with both code and message.
    pub fn with_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: Some(code.into()),
            message: message.into(),
        }
    }

    /// Creates an error from any displayable value.
    pub fn from_display(err: impl Display) -> Self {
        Self::new(err.to_string())
    }
}

impl From<String> for HostAbiError {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

impl From<&str> for HostAbiError {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
/// Top-level response envelope used by host ABI exports.
pub struct HostAbiResponse {
    pub success: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty", with = "serde_bytes")]
    pub payload: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl HostAbiResponse {
    /// Builds a successful response with encoded payload bytes.
    pub fn ok(payload: Vec<u8>) -> Self {
        Self {
            success: true,
            payload,
            error_code: None,
            error_message: None,
        }
    }

    /// Builds an error response from any value convertible to [`HostAbiError`].
    pub fn err(error: impl Into<HostAbiError>) -> Self {
        let error = error.into();
        Self {
            success: false,
            payload: Vec::new(),
            error_code: error.code,
            error_message: Some(error.message),
        }
    }
}

/// Packs `(ptr, len)` into a single `u64` where the high 32 bits hold length.
pub fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((len as u64) << 32) | (ptr as u64)
}

/// Unpacks a pointer/length pair produced by [`pack_ptr_len`].
pub fn unpack_ptr_len(packed: u64) -> (u32, u32) {
    let ptr = packed as u32;
    let len = (packed >> 32) as u32;
    (ptr, len)
}

/// Allocates `len` bytes in wasm linear memory and returns pointer.
pub fn alloc(len: u32) -> u32 {
    if len == 0 {
        return 0;
    }

    let mut buffer = Vec::<u8>::with_capacity(len as usize);
    let ptr = buffer.as_mut_ptr();
    std::mem::forget(buffer);
    ptr as u32
}

/// Frees memory previously allocated via `alloc`.
///
/// # Safety
/// The `(ptr, len)` pair must come from `alloc` and must not be freed twice.
pub unsafe fn free(ptr: u32, len: u32) {
    if ptr == 0 || len == 0 {
        return;
    }

    // SAFETY: Caller guarantees the pointer/length pair originated from `alloc`.
    unsafe {
        drop(Vec::from_raw_parts(
            ptr as *mut u8,
            len as usize,
            len as usize,
        ));
    }
}

/// Reads a byte slice from wasm linear memory.
pub fn read_memory(ptr: u32, len: u32) -> Result<Vec<u8>, HostAbiError> {
    if len == 0 {
        return Ok(Vec::new());
    }
    if ptr == 0 {
        return Err(HostAbiError::with_code(
            ERR_INVALID_POINTER,
            "received null pointer for non-empty input",
        ));
    }

    // SAFETY: The host is expected to pass a valid input buffer in wasm memory.
    let bytes = unsafe { std::slice::from_raw_parts(ptr as *const u8, len as usize) };
    Ok(bytes.to_vec())
}

/// Writes bytes into wasm linear memory and returns a packed pointer/length pair.
pub fn write_memory(data: &[u8]) -> u64 {
    let len = data.len() as u32;
    if len == 0 {
        return pack_ptr_len(0, 0);
    }

    let ptr = alloc(len);
    if ptr == 0 {
        return pack_ptr_len(0, 0);
    }

    // SAFETY: `ptr` points to `len` bytes allocated via `alloc`.
    unsafe {
        std::ptr::copy_nonoverlapping(data.as_ptr(), ptr as *mut u8, len as usize);
    }
    pack_ptr_len(ptr, len)
}

/// Deserialize CBOR bytes into a value.
pub fn cbor_from_slice<T: serde::de::DeserializeOwned>(bytes: &[u8]) -> Result<T, HostAbiError> {
    ciborium::de::from_reader(bytes)
        .map_err(|e| HostAbiError::with_code(ERR_INVALID_CBOR, e.to_string()))
}

/// Serialize a value to CBOR bytes.
pub fn cbor_to_vec<T: serde::Serialize>(value: &T) -> Result<Vec<u8>, HostAbiError> {
    let mut buf = Vec::new();
    ciborium::ser::into_writer(value, &mut buf)
        .map_err(|e| HostAbiError::with_code(ERR_SERIALIZATION_FAILED, e.to_string()))?;
    Ok(buf)
}

/// Serializes [`HostAbiResponse`] to CBOR and writes it into wasm memory.
pub fn encode_response(response: &HostAbiResponse) -> u64 {
    let encoded = cbor_to_vec(response).unwrap_or_else(|error| {
        let fallback = HostAbiResponse::err(HostAbiError::with_code(
            ERR_SERIALIZATION_FAILED,
            error.message,
        ));
        cbor_to_vec(&fallback).unwrap_or_default()
    });
    write_memory(&encoded)
}

/// Reads input bytes, invokes a handler, and returns an encoded ABI response.
pub fn dispatch<E, F>(ptr: u32, len: u32, handler: F) -> u64
where
    E: Into<HostAbiError>,
    F: FnOnce(&[u8]) -> Result<Vec<u8>, E> + UnwindSafe,
{
    let response = dispatch_response(read_memory(ptr, len), handler);
    encode_response(&response)
}

/// Converts handler output (or failure) into [`HostAbiResponse`].
pub fn dispatch_response<E, F>(input: Result<Vec<u8>, HostAbiError>, handler: F) -> HostAbiResponse
where
    E: Into<HostAbiError>,
    F: FnOnce(&[u8]) -> Result<Vec<u8>, E> + UnwindSafe,
{
    match input {
        Ok(input) => match std::panic::catch_unwind(|| handler(&input)) {
            Ok(handler_result) => match handler_result {
                Ok(payload) => HostAbiResponse::ok(payload),
                Err(error) => HostAbiResponse::err(error.into()),
            },
            Err(_) => HostAbiResponse::err(HostAbiError::with_code(
                ERR_HANDLER_PANIC,
                "handler panicked",
            )),
        },
        Err(error) => HostAbiResponse::err(error),
    }
}

#[macro_export]
macro_rules! ownable_host_abi_v1 {
    (
        instantiate = $instantiate:path,
        execute = $execute:path,
        query = $query:path,
        external_event = $external_event:path $(,)?
    ) => {
        #[unsafe(no_mangle)]
        pub extern "C" fn ownable_alloc(len: u32) -> u32 {
            $crate::abi::alloc(len)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn ownable_free(ptr: u32, len: u32) {
            // SAFETY: Host must only pass pointers obtained through `ownable_alloc`.
            unsafe { $crate::abi::free(ptr, len) }
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn ownable_instantiate(ptr: u32, len: u32) -> u64 {
            $crate::abi::dispatch(ptr, len, $instantiate)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn ownable_execute(ptr: u32, len: u32) -> u64 {
            $crate::abi::dispatch(ptr, len, $execute)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn ownable_query(ptr: u32, len: u32) -> u64 {
            $crate::abi::dispatch(ptr, len, $query)
        }

        #[unsafe(no_mangle)]
        pub extern "C" fn ownable_external_event(ptr: u32, len: u32) -> u64 {
            $crate::abi::dispatch(ptr, len, $external_event)
        }
    };
}

/// A single key-value attribute from a contract response.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbiAttribute {
    pub key: String,
    pub value: String,
}

/// An event emitted by a contract response.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbiEvent {
    #[serde(rename = "type")]
    pub kind: String,
    pub attributes: Vec<AbiAttribute>,
}

/// CBOR-native representation of a cosmwasm execute/instantiate/external_event Response.
/// Only carries the fields the host actually needs; skips messages and sub-messages.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbiResponse {
    pub attributes: Vec<AbiAttribute>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub events: Vec<AbiEvent>,
}

impl From<Response> for AbiResponse {
    fn from(r: Response) -> Self {
        AbiResponse {
            attributes: r
                .attributes
                .into_iter()
                .map(|a| AbiAttribute {
                    key: a.key,
                    value: a.value,
                })
                .collect(),
            events: r
                .events
                .into_iter()
                .map(|e| AbiEvent {
                    kind: e.ty,
                    attributes: e
                        .attributes
                        .into_iter()
                        .map(|a| AbiAttribute {
                            key: a.key,
                            value: a.value,
                        })
                        .collect(),
                })
                .collect(),
        }
    }
}

/// The inner payload returned by every ABI handler, CBOR-encoded inside `HostAbiResponse.payload`.
///
/// - `result` carries raw bytes:
///   - for execute/instantiate/external_event: a CBOR-encoded `AbiResponse`
///   - for query: the raw bytes from cosmwasm `Binary` (JSON-encoded by `to_json_binary`)
/// - `mem` is present for state-mutating calls; absent for queries.
#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct AbiResultPayload {
    #[serde(with = "serde_bytes")]
    pub result: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mem: Option<IdbStateDump>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn pack_unpack_round_trip() {
        let packed = pack_ptr_len(42, 128);
        assert_eq!(unpack_ptr_len(packed), (42, 128));
    }

    #[test]
    fn response_serializes_error_fields() {
        let response = HostAbiResponse::err(HostAbiError::with_code("E", "failed"));
        let encoded = cbor_to_vec(&response).expect("serialize");
        let decoded: HostAbiResponse = cbor_from_slice(&encoded).expect("deserialize");
        assert!(!decoded.success);
        assert_eq!(decoded.error_code.as_deref(), Some("E"));
        assert_eq!(decoded.error_message.as_deref(), Some("failed"));
    }

    #[test]
    fn response_omits_empty_payload() {
        let response = HostAbiResponse::err("boom");
        let encoded = cbor_to_vec(&response).expect("serialize");
        let decoded: HostAbiResponse = cbor_from_slice(&encoded).expect("deserialize");
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn payload_round_trips_as_cbor_bytes_not_array() {
        // Ensure Vec<u8> payload is encoded as CBOR byte string (major type 2),
        // not a CBOR array of integers (major type 4). cbor-x on the JS side
        // decodes byte strings to Uint8Array; it cannot decode a CBOR array as
        // input to a second decode() call.
        let inner = vec![0x01u8, 0x02, 0x03];
        let response = HostAbiResponse::ok(inner.clone());
        let encoded = cbor_to_vec(&response).expect("serialize");

        // The "payload" value in the CBOR map must be a byte string (major type 2),
        // NOT an array (major type 4).
        let value: ciborium::Value =
            ciborium::de::from_reader(&encoded[..]).expect("parse as Value");
        if let ciborium::Value::Map(entries) = value {
            let payload_val = entries
                .into_iter()
                .find(|(k, _)| k == &ciborium::Value::Text("payload".into()))
                .map(|(_, v)| v)
                .expect("payload key present");
            assert!(
                matches!(payload_val, ciborium::Value::Bytes(_)),
                "payload must be CBOR bytes, got {:?}",
                payload_val
            );
        } else {
            panic!("expected CBOR map");
        }

        // Also verify round-trip correctness.
        let decoded: HostAbiResponse = cbor_from_slice(&encoded).expect("deserialize");
        assert_eq!(decoded.payload, inner);
    }

    #[test]
    fn cbor_parse_error_maps_to_invalid_cbor_code() {
        let result: Result<HostAbiResponse, _> = cbor_from_slice(b"\xff");
        let abi_err = result.expect_err("should fail on invalid CBOR");
        assert_eq!(abi_err.code.as_deref(), Some(ERR_INVALID_CBOR));
    }

    #[test]
    fn dispatch_converts_panic_into_structured_error() {
        let response = dispatch_response::<HostAbiError, _>(
            Ok(Vec::new()),
            |_| -> Result<Vec<u8>, HostAbiError> {
                panic!("boom");
            },
        );
        assert!(!response.success);
        assert_eq!(response.error_code.as_deref(), Some(ERR_HANDLER_PANIC));
        assert_eq!(response.error_message.as_deref(), Some("handler panicked"));
    }
}
