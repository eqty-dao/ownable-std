# ownable-std

`ownable-std` is the shared Rust standard library for Ownables contracts.

It provides:
- common types/utilities used by Ownables contracts
- optional message-shaping proc macros (`ownable-std-macros`)
- a stable Host ABI v1 for wasm runtime calls that does not depend on `wasm-bindgen` JS glue compatibility
- split external-event types and helpers for public events and cross-ownable events

## Install

```toml
[dependencies]
ownable-std = "0.7.0"
```

`macros` is enabled by default. To disable proc macros:

```toml
[dependencies]
ownable-std = { version = "0.7.0", default-features = false }
```

`ownable-std` has no `wasm-bindgen` or `js-sys` dependency in runtime contract usage.

## Host ABI v1

Constants:
- `ownable_std::abi::HOST_ABI_VERSION` = `"1"`
- `ownable_std::abi::HOST_ABI_MANIFEST_FIELD` = `"ownablesAbi"`
- `ownable_std::abi::HOST_ABI_WIRE_FORMAT` = `"cbor"`
- `ownable_std::abi::HOST_ABI_WIRE_FORMAT_MANIFEST_FIELD` = `"wireFormat"`

Stable exports:
- `ownable_alloc(len: u32) -> u32`
- `ownable_free(ptr: u32, len: u32)`
- `ownable_instantiate(ptr: u32, len: u32) -> u64`
- `ownable_execute(ptr: u32, len: u32) -> u64`
- `ownable_query(ptr: u32, len: u32) -> u64`
- `ownable_register(ptr: u32, len: u32) -> u64`
- `ownable_ingest(ptr: u32, len: u32) -> u64`
- `ownable_encode_public_event(ptr: u32, len: u32) -> u64`

Return strategy:
- entrypoints return packed `u64`
- low 32 bits = output pointer
- high 32 bits = output length

Wire format:
- input: CBOR bytes
- output: CBOR envelope bytes

### Request Schemas (Per Call)

At the ABI transport layer, three calls use contract-defined CBOR payloads:
- `instantiate`: CBOR document bytes
- `execute`: CBOR document bytes
- `query`: CBOR document bytes

`register` uses a standardized CBOR-encoded `ownable_std::register::PublicEvent` payload.

`ingest` uses a standardized CBOR-encoded `ownable_std::ingest::OwnableEvent` payload.

`encode_public_event` uses a standardized CBOR-encoded
`ownable_std::register::EncodePublicEventRequest` payload and returns raw ABI bytes.

Required ABI-level keys:
- none (the full request shape is contract-defined)

Recommended convention for cross-contract consistency:

```text
{
  abi_version: "1",
  payload: <contract-defined value>
}
```

If you use the convention above:
- `abi_version`: required string, must be `"1"`
- `payload`: required contract-defined CBOR value

Response envelope schema (decoded structure):

```text
{
  success: bool,
  payload: bytes,
  error_code: string | null,
  error_message: string | null
}
```

## Contract Usage

Use the `ownable_host_abi_v1!` macro to export all required ABI symbols.

```rust
use ownable_std::abi::HostAbiError;
use ownable_std::ownable_host_abi_v1;

fn instantiate_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    let v: serde_cbor::Value = serde_cbor::from_slice(input)?;
    serde_cbor::to_vec(&v).map_err(HostAbiError::from)
}

fn execute_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    let v: serde_cbor::Value = serde_cbor::from_slice(input)?;
    serde_cbor::to_vec(&v).map_err(HostAbiError::from)
}

fn query_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    let v: serde_cbor::Value = serde_cbor::from_slice(input)?;
    serde_cbor::to_vec(&v).map_err(HostAbiError::from)
}

fn register_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    let event: ownable_std::register::PublicEvent = ownable_std::abi::cbor_from_slice(input)?;
    let response = match event.event_type.as_str() {
        "consume" => {
            type ConsumeEvent = alloy_sol_types::sol!((address consumer, uint256 amount));
            let (_consumer, _amount) =
                ownable_std::register::decode_abi_for::<ConsumeEvent>(&event, "consume")?;
            ownable_std::abi::AbiResponse {
                attributes: vec![],
                events: vec![],
            }
        }
        other => {
            return Err(HostAbiError::new(format!(
                "unsupported public event type: {other}"
            )));
        }
    };

    ownable_std::abi::cbor_to_vec(&response).map_err(HostAbiError::from)
}

fn ingest_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    let event: ownable_std::ingest::OwnableEvent =
        ownable_std::abi::cbor_from_slice(input)?;
    ownable_std::ingest::require_ownable_event_type(&event, "consume")
        .map_err(HostAbiError::from_display)?;

    let amount = event
        .attributes
        .get("amount")
        .and_then(|value| value.as_u64())
        .ok_or_else(|| HostAbiError::new("missing amount attribute"))?;

    let response = ownable_std::abi::AbiResponse {
        attributes: vec![ownable_std::abi::AbiAttribute {
            key: "amount".to_string(),
            value: amount.to_string(),
        }],
        events: vec![],
    };

    ownable_std::abi::cbor_to_vec(&response).map_err(HostAbiError::from)
}

fn encode_public_event_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    let request: ownable_std::register::EncodePublicEventRequest =
        ownable_std::abi::cbor_from_slice(input)?;

    match request.event_type.as_str() {
        "consume" => {
            #[derive(serde::Deserialize)]
            struct ConsumePayload {
                amount: u32,
                active: bool,
            }

            type ConsumeEvent = alloy_sol_types::sol!((uint32, bool));

            let payload: ConsumePayload =
                ownable_std::abi::cbor_from_slice(request.data.as_slice())?;
            Ok(ownable_std::register::encode_abi::<ConsumeEvent>(&(
                payload.amount,
                payload.active,
            )))
        }
        other => Err(HostAbiError::new(format!(
            "unsupported public event type: {other}"
        ))),
    }
}

ownable_host_abi_v1!(
    instantiate = instantiate_handler,
    execute = execute_handler,
    query = query_handler,
    register = register_handler,
    ingest = ingest_handler,
    encode_public_event = encode_public_event_handler,
);
```

Handler signature expected by the macro:

```rust
fn handler(input: &[u8]) -> Result<Vec<u8>, E>
where
    E: Into<ownable_std::abi::HostAbiError>;
```

## Host Runtime Call Flow

For each call (`instantiate`, `execute`, `query`, `register`, `ingest`, `encode_public_event`):
1. Serialize request object to CBOR bytes.
2. Call `ownable_alloc(len)`.
3. Write bytes into wasm memory at the returned pointer.
4. Call the selected `ownable_*` entrypoint with `(ptr, len)`.
5. Unpack `(out_ptr, out_len)` from returned `u64`.
6. Read `out_len` bytes at `out_ptr`.
7. Parse CBOR envelope and check `success`.
8. Call `ownable_free(out_ptr, out_len)`.

### Memory Ownership Rules

- `ownable_alloc(len)` allocates wasm-side memory for host writes.
- Host owns writing input bytes into this allocated buffer.
- Entrypoints consume input bytes immediately and copy them internally; host may treat input buffer as transient.
- Entrypoints allocate output envelope bytes and return `(ptr,len)` packed in `u64`.
- Host must call `ownable_free(out_ptr, out_len)` exactly once for every non-zero output buffer.
- `ptr=0,len=0` means empty output.
- Passing `ptr=0,len>0` to entrypoints yields structured ABI error (`INVALID_POINTER`), not panic.
- This crate does not enforce a hard max payload size; host/runtime should enforce practical limits.

### Error Code Catalog

Current ABI-level `error_code` values:
- `INVALID_POINTER`: null pointer with non-zero length input
- `INVALID_CBOR`: request bytes failed CBOR parsing in contract handler
- `SERIALIZATION_FAILED`: ABI envelope serialization failed
- `HANDLER_PANIC`: contract handler panicked; panic is converted to structured error

Handlers may return additional domain-specific error codes via `HostAbiError::with_code`.

### Non-UTF8 Behavior

Non-UTF8 input must not panic.

Expected behavior:
- handler CBOR parse fails (`serde_cbor::from_slice`)
- error is mapped to structured ABI error envelope (`success=false`, `error_code=INVALID_CBOR`)

## Building Ownables

Build command:

```bash
cargo build --target wasm32-unknown-unknown --release
```

This produces a wasm module consumable by a host/runtime that implements Host ABI v1.

Contract-side requirements:
- do not export runtime entrypoints with `#[wasm_bindgen]`
- do not rely on generated `*.js` glue for host calls
- export only the stable `ownable_*` ABI symbols (via `ownable_host_abi_v1!`)

## End-to-End Test Vector

Example contract handler:

```rust
fn instantiate_handler(input: &[u8]) -> Result<Vec<u8>, ownable_std::abi::HostAbiError> {
    Ok(input.to_vec())
}
```

Host request bytes (CBOR, decoded form shown):

```text
{ ping: "pong" }
```

Expected output envelope bytes decode to the same payload bytes.

```text
{
  success: true,
  payload: <CBOR bytes for { ping: "pong" }>
}
```

## Other Utilities in This Crate

- `MemoryStorage`: in-memory storage implementation for testing/off-chain execution
- `create_env` / `create_ownable_env`: env builders
- `register` / `ingest`: public and cross-ownable event transport types plus helpers
- `EncodePublicEventRequest`: JS-to-wasm request payload for public-event ABI encoding
- `package_title_from_name`, color helpers, metadata/shared message structs
- `ownable-std-macros`: attribute macros to extend execute/query/instantiate messages

## Versioning Guidance

- set your package manifest field `ownablesAbi` to `"1"` for exported symbol/signature ABI compatibility
- set your package manifest field `wireFormat` to `"cbor"` for payload envelope encoding compatibility
- host/runtime should reject incompatible versions
- non-interface dependency changes should not require runtime changes if ABI version stays the same
