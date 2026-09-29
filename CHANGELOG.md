# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.6] - 2026-09-29
### Fixed
- Reject an empty prefix
- Prefix `HTTPServer` as `TestHTTPServer`, not `TestHttpserver`
- Reference a nested `TestDRemote`, not `TestDremote`
- Document the generated macro, so that a crate with `#![deny(missing_docs)]` compiles
- Report `clappen must be used on mod only` when `#[clappen]` is not on a module

## [0.1.5] - 2026-09-16
### Fixed
- List qualified `apply` paths in the generated nested-struct doc

## [0.1.4] - 2026-07-01
### Fixed
- Preserve trait, generics and `unsafe` when prefixing impls
- Docstring and README typos

## [0.1.3] - 2024-12-14
### Fixed
- Add missing licenses

## [0.1.2] - 2024-12-12
### Fixed
- Readme examples cleanup
- Ensure snake_case prefix on nested structs fields

## [0.1.1] - 2024-12-09
### Fixed
- Fix readme links

## [0.1.0] - 2024-12-09
### Added
- Initial version
