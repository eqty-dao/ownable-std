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

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HostAbiError {
    pub code: Option<String>,
    pub message: String,
}

impl HostAbiError {
    pub fn new(message: impl Into<String>) -> Self {
        Self {
            code: None,
            message: message.into(),
        }
    }

    pub fn with_code(code: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            code: Some(code.into()),
            message: message.into(),
        }
    }

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

impl From<serde_cbor::Error> for HostAbiError {
    fn from(value: serde_cbor::Error) -> Self {
        Self::with_code(ERR_INVALID_CBOR, value.to_string())
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq)]
pub struct HostAbiResponse {
    pub success: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub payload: Vec<u8>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error_message: Option<String>,
}

impl HostAbiResponse {
    pub fn ok(payload: Vec<u8>) -> Self {
        Self {
            success: true,
            payload,
            error_code: None,
            error_message: None,
        }
    }

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

pub fn pack_ptr_len(ptr: u32, len: u32) -> u64 {
    ((len as u64) << 32) | (ptr as u64)
}

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

pub fn encode_response(response: &HostAbiResponse) -> u64 {
    let encoded = serde_cbor::to_vec(response).unwrap_or_else(|error| {
        let fallback = HostAbiResponse::err(HostAbiError::with_code(
            ERR_SERIALIZATION_FAILED,
            error.to_string(),
        ));
        serde_cbor::to_vec(&fallback).unwrap_or_else(|_| Vec::new())
    });
    write_memory(&encoded)
}

pub fn dispatch<E, F>(ptr: u32, len: u32, handler: F) -> u64
where
    E: Into<HostAbiError>,
    F: FnOnce(&[u8]) -> Result<Vec<u8>, E> + UnwindSafe,
{
    let response = dispatch_response(read_memory(ptr, len), handler);
    encode_response(&response)
}

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
        let encoded = serde_cbor::to_vec(&response).expect("serialize");
        let decoded: HostAbiResponse = serde_cbor::from_slice(&encoded).expect("deserialize");
        assert!(!decoded.success);
        assert_eq!(decoded.error_code.as_deref(), Some("E"));
        assert_eq!(decoded.error_message.as_deref(), Some("failed"));
    }

    #[test]
    fn response_omits_empty_payload() {
        let response = HostAbiResponse::err("boom");
        let encoded = serde_cbor::to_vec(&response).expect("serialize");
        let decoded: HostAbiResponse = serde_cbor::from_slice(&encoded).expect("deserialize");
        assert!(decoded.payload.is_empty());
    }

    #[test]
    fn serde_cbor_error_maps_to_invalid_cbor_code() {
        let err = serde_cbor::from_slice::<HostAbiResponse>(b"\xff").expect_err("invalid");
        let abi_err: HostAbiError = err.into();
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
