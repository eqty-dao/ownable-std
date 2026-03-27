# ownable-std

Library for Ownables smart contracts development

## Host ABI v1

`ownable-std` now exposes a stable host ABI surface that is independent from
`wasm-bindgen` JS glue internals.

- ABI version constant: `ownable_std::abi::HOST_ABI_VERSION` (`"1"`)
- Suggested manifest field: `ownable_std::abi::HOST_ABI_MANIFEST_FIELD` (`"ownablesAbi"`)
- Stable exported symbols:
  - `ownable_alloc(len: u32) -> u32`
  - `ownable_free(ptr: u32, len: u32)`
  - `ownable_instantiate(ptr: u32, len: u32) -> u64`
  - `ownable_execute(ptr: u32, len: u32) -> u64`
  - `ownable_query(ptr: u32, len: u32) -> u64`
  - `ownable_external_event(ptr: u32, len: u32) -> u64`

The return value for entrypoints is packed as `u64`:

- lower 32 bits: output pointer
- upper 32 bits: output length

Response wire envelope (`UTF-8 JSON bytes`) is:

```json
{
  "success": true,
  "payload": [123, 34, 111, 107, 34, 58, 116, 114, 117, 101, 125]
}
```

or

```json
{
  "success": false,
  "error_code": "SOME_CODE",
  "error_message": "Human readable message"
}
```

### Contract-side export macro

Use `ownable_host_abi_v1!` to export all stable symbols:

```rust
use ownable_std::ownable_host_abi_v1;
use ownable_std::abi::HostAbiError;

fn instantiate_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    Ok(input.to_vec())
}

fn execute_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    Ok(input.to_vec())
}

fn query_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    Ok(input.to_vec())
}

fn external_event_handler(input: &[u8]) -> Result<Vec<u8>, HostAbiError> {
    Ok(input.to_vec())
}

ownable_host_abi_v1!(
    instantiate = instantiate_handler,
    execute = execute_handler,
    query = query_handler,
    external_event = external_event_handler,
);
```

### Host call flow

1. Encode request JSON as bytes.
2. Call `ownable_alloc(len)`.
3. Write request bytes into wasm memory at `ptr`.
4. Call one of `ownable_instantiate/execute/query/external_event`.
5. Unpack `(out_ptr, out_len)` from returned `u64`.
6. Read output bytes from wasm memory and parse JSON envelope.
7. Call `ownable_free(out_ptr, out_len)` to release output buffer.
