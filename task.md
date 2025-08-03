# Fix State Management

**Status:** InProgress
**Agent PID:** 19962

## Original Todo

## 3. Fix State Management

**Priority:** Medium  
**Goal:** Remove environmental concerns from domain state.

### Steps

1. Remove `vault_path` from `SessionMetadata` struct
2. Update all functions that construct `SessionMetadata`
3. Ensure `EffectRunner` is the sole holder of `vault_path`
4. Update serialization/deserialization to exclude vault path
5. Fix all compilation errors from removed field

### Testing Strategy

- Verify session persistence works without vault_path
- Test session loading from different vault paths
- Ensure session state is purely domain-focused
- Verify no environmental data leaks into saved sessions

## Description

We're removing the `vault_path` field from the `SessionMetadata` struct to properly separate environmental concerns from domain state. Currently, the vault path is inappropriately stored in the session's domain data and gets persisted to disk, creating coupling between the business logic and infrastructure. The `EffectRunner` already manages the vault path for all I/O operations, making the field in `SessionMetadata` redundant and architecturally incorrect.

## Implementation Plan

Based on the codebase analysis, here's how we'll remove `vault_path` from the domain state:

- [ ] Remove `vault_path` field from `SessionMetadata` struct in src/journal/src/state.rs:44-50
- [ ] Update `JournalSession::new()` signature to remove vault_path parameter in src/journal/src/state.rs:105-111
- [ ] Fix session creation in src/journal/src/update.rs:20-25 to not pass vault_path
- [ ] Update test session creation in src/journal/src/update.rs:199-200, 224-225
- [ ] Verify session serialization/deserialization works without vault_path in src/journal/src/effects.rs:168-231
- [ ] Run `cargo build` and fix any remaining compilation errors
- [ ] Run `cargo test` to ensure all tests pass
- [ ] Run `cargo clippy` to check for any linting issues
- [ ] User test: Create a new journal session and verify it saves/loads correctly without vault_path

## Notes

[Implementation notes]