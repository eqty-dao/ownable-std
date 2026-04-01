# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.6.1](https://github.com/eqty-dao/ownable-std/compare/ownable-std-v0.6.0...ownable-std-v0.6.1) - 2026-04-01

### Fixed

- *(ci)* pin macros dependency version during release publish

### Other

- Revert back to path dep during development

## [0.6.0](https://github.com/eqty-dao/ownable-std/compare/ownable-std-v0.5.0...ownable-std-v0.6.0) - 2026-04-01

### Added

- add ensure_owner helper for owner authorization

### Fixed

- use path-only macros dependency for release-plz update

### Other

- Don't emphesise on no wasm_bindgen in README

## [0.5.0](https://github.com/eqty-dao/ownable-std/compare/v0.4.0...v0.5.0) - 2026-03-31

### Added

- stable host ABI v1 exports for ownable wasm entry points
- CBOR-native host ABI request/response handling and payload encoding
- explicit ABI compatibility constants including wire-format markers
- structured ABI error model with handler panic/serialization/CBOR error codes
- ABI response conversion types for CosmWasm `Response` data

### Changed

- host wire format switched from JSON/serde_json interop to CBOR (`ciborium`)
- removed bindgen runtime interop path from `ownable-std` runtime
- `ownable-std` and `ownable-std-macros` now released in lockstep version groups
- procedural macros migrated to `syn` v2

### Documentation

- expanded ABI specification and runtime semantics in README
- added rustdoc (`///`) for core public APIs in `ownable-std` and `ownable-std-macros`
- added tests workflow for workspace unit tests on GitHub Actions

### Fixed

- fix release-plz config parsing by moving `release_always` to workspace level

## [0.4.0](https://github.com/eqty-dao/ownable-std/compare/v0.3.1...v0.4.0) - 2026-03-26

### Added

- migrate to cosmwasm-std v3 with crate-owned memory storage
- add package_title_from_name utility

### Other

- trigger release only after merged release-plz PR
- restore release-pr workflow for automatic version bumps
- tighten release workflow permissions
- simplify release-plz workflow to single release job
- add release-plz automated crates.io release workflow
- add unit tests for ownable-std utility functions
