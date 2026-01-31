# CHANGELOG

All significant changes to this project will be documented in this file.

## Unreleased

## v0.2.0 (2026-01-31)

### Breaking Changes

* `FastraceLayer` now requires explicit construction: `FastraceLayer::default()`.

### New Features

* Added a configurable span context extractor; default reads `traceparent` and starts a new trace when missing or invalid.
