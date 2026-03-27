# ownable-std

`ownable-std` is the shared Rust standard library for Ownables contracts.

It provides:
- common types/utilities used by Ownables contracts
- optional message-shaping proc macros (`ownable-std-macros`)
- a stable Host ABI v1 for wasm runtime calls that does not depend on `wasm-bindgen` JS glue compatibility

## Why Host ABI v1

Historically, browser/runtime integration could fail when wasm output and `wasm-bindgen` JS glue drifted.

Host ABI v1 fixes the contract boundary by using fixed exported symbols and a bytes-in/bytes-out protocol:
- no hash-derived bindgen symbol coupling
- no requirement to trust package-provided generated JS glue
- runtime can call any contract that keeps the same ABI version

## Install

```toml
[dependencies]
ownable-std = "0.4.0"
```

`macros` is enabled by default. To disable proc macros:

```toml
[dependencies]
ownable-std = { version = "0.4.0", default-features = false }
```

## Host ABI v1

Constants:
- `ownable_std::abi::HOST_ABI_VERSION` = `"1"`
- `ownable_std::abi::HOST_ABI_MANIFEST_FIELD` = `"ownablesAbi"`

Stable exports:
- `ownable_alloc(len: u32) -> u32`
- `ownable_free(ptr: u32, len: u32)`
- `ownable_instantiate(ptr: u32, len: u32) -> u64`
- `ownable_execute(ptr: u32, len: u32) -> u64`
- `ownable_query(ptr: u32, len: u32) -> u64`
- `ownable_external_event(ptr: u32, len: u32) -> u64`

Return strategy:
- entrypoints return packed `u64`
- low 32 bits = output pointer
- high 32 bits = output length

Wire format:
- input: UTF-8 JSON bytes
- output: UTF-8 JSON envelope bytes

### Request Schemas (Per Call)

At the ABI transport layer, all four calls use the same schema:
- `instantiate`: JSON document bytes
- `execute`: JSON document bytes
- `query`: JSON document bytes
- `external_event`: JSON document bytes

Required ABI-level keys:
- none (the full request shape is contract-defined)

Recommended convention for cross-contract consistency:

```json
{
  "abi_version": "1",
  "payload": {}
}
```

If you use the convention above:
- `abi_version`: required string, must be `"1"`
- `payload`: required JSON value/object defined by your contract

Response envelope schema:

```json
{
  "success": true,
  "payload": [123, 34, 111, 107, 34, 58, 116, 114, 117, 101, 125]
}
```

Error envelope example:

```json
{
  "success": false,
  "error_code": "SOME_CODE",
  "error_message": "Human readable message"
}
```

## Contract Usage (No `wasm_bindgen`)

Use the `ownable_host_abi_v1!` macro to export all required ABI symbols.

```rust
use ownable_std::abi::HostAbiError;
use ownable_std::ownable_host_abi_v1;
use serde_json::json;

fn instantiate_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    // Parse your request bytes, execute logic, return response bytes.
    let v: serde_json::Value = serde_json::from_slice(input)?;
    Ok(serde_json::to_vec(&json!({ "kind": "instantiate", "input": v }))?)
}

fn execute_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    let v: serde_json::Value = serde_json::from_slice(input)?;
    Ok(serde_json::to_vec(&json!({ "kind": "execute", "input": v }))?)
}

fn query_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    let v: serde_json::Value = serde_json::from_slice(input)?;
    Ok(serde_json::to_vec(&json!({ "kind": "query", "input": v }))?)
}

fn external_event_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    let v: serde_json::Value = serde_json::from_slice(input)?;
    Ok(serde_json::to_vec(&json!({ "kind": "external_event", "input": v }))?)
}

ownable_host_abi_v1!(
    instantiate = instantiate_handler,
    execute = execute_handler,
    query = query_handler,
    external_event = external_event_handler,
);
```

Handler signature expected by the macro:

```rust
fn handler(input: &[u8]) -> Result<Vec<u8>, E>
where
    E: Into<ownable_std::abi::HostAbiError>;
```

## Host Runtime Call Flow

For each call (`instantiate`, `execute`, `query`, `external_event`):
1. Serialize request object to UTF-8 JSON bytes.
2. Call `ownable_alloc(len)`.
3. Write bytes into wasm memory at the returned pointer.
4. Call the selected `ownable_*` entrypoint with `(ptr, len)`.
5. Unpack `(out_ptr, out_len)` from returned `u64`.
6. Read `out_len` bytes at `out_ptr`.
7. Parse JSON envelope and check `success`.
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
- `INVALID_JSON`: request bytes failed JSON parsing in contract handler
- `SERIALIZATION_FAILED`: ABI envelope serialization failed
- `HANDLER_PANIC`: contract handler panicked; panic is converted to structured error

Handlers may return additional domain-specific error codes via `HostAbiError::with_code`.

### Non-UTF8 Behavior

Non-UTF8 input must not panic.

Expected behavior:
- handler JSON parse fails (`serde_json::from_slice`)
- error is mapped to structured ABI error envelope (`success=false`, `error_code=INVALID_JSON`)

## Building Ownables Without `wasm_bindgen`

Contract-side requirements:
- do not export runtime entrypoints with `#[wasm_bindgen]`
- do not rely on generated `*.js` glue for host calls
- export only the stable `ownable_*` ABI symbols (via `ownable_host_abi_v1!`)

Build command:

```bash
cargo build --target wasm32-unknown-unknown --release
```

This produces a wasm module consumable by a host/runtime that implements Host ABI v1.

## End-to-End Test Vector

Example contract handler:

```rust
fn instantiate_handler(input: &[u8]) -> Result<Vec<u8>, ownable_std::abi::HostAbiError> {
    Ok(input.to_vec())
}
```

Host request bytes (UTF-8 JSON):

```json
{"ping":"pong"}
```

Expected output envelope bytes decode to:

```json
{
  "success": true,
  "payload": [123,34,112,105,110,103,34,58,34,112,111,110,103,34,125]
}
```

## Other Utilities in This Crate

- `MemoryStorage`: in-memory storage implementation for testing/off-chain execution
- `create_env` / `create_ownable_env`: env builders
- `package_title_from_name`, color helpers, metadata/shared message structs
- `ownable-std-macros`: attribute macros to extend execute/query/instantiate messages

## Versioning Guidance

- set your package manifest field `ownablesAbi` to `"1"` when using this ABI
- host/runtime should reject incompatible versions
- non-interface dependency changes should not require runtime changes if ABI version stays the same
