# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
