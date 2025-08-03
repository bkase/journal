# Fix State Management

**Status:** Refining
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

[what we're building]

## Implementation Plan

[how we are building it]

- [ ] Code change with location(s) if applicable (src/file.ts:45-93)
- [ ] Automated test: ...
- [ ] User test: ...

## Notes

[Implementation notes]