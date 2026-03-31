# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.5.0](https://github.com/eqty-dao/ownable-std/compare/v0.4.0...v0.5.0) - 2026-03-31

### Added

- add CI workflow to run workspace unit tests on GitHub Actions
- add rustdoc (`///`) comments to main public APIs in `ownable-std` and `ownable-std-macros`

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
