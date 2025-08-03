# Todos

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

## 4. Improve Error Handling

**Priority:** Medium  
**Goal:** Use typed errors for better error discrimination and handling.

### Steps

1. Add `thiserror` dependency to `Cargo.toml`
2. Create `effects::Error` enum:

   ```rust
   #[derive(Error, Debug)]
   pub enum Error {
       #[error("Aethel operation failed")]
       Aethel(#[from] aethel_core::Error),
       #[error("IO operation failed")]
       Io(#[from] std::io::Error),
       #[error("AI analysis failed")]
       AiAnalysis(String),
       #[error("Session not found")]
       SessionNotFound,
       #[error("Invalid session state")]
       InvalidSessionState,
   }
   ```

3. Update `execute()` to return `Result<Action, Error>`
4. Update main loop to match on specific errors
5. Create appropriate error-to-action mappings
6. Remove string-based error handling

### Testing Strategy

- Test each error variant produces appropriate action
- Verify error context is preserved
- Test fallback behaviors for AI analysis failures
- Ensure error messages are still user-friendly

## 5. Clean Data Structures

**Priority:** Low  
**Goal:** Improve data serialization maintainability.

### Steps

1. Create `SessionFrontmatter` struct:

   ```rust
   #[derive(Serialize, Deserialize)]
   pub struct SessionFrontmatter {
       pub id: String,
       pub title: Option<String>,
       pub created_at: DateTime<Utc>,
       pub updated_at: DateTime<Utc>,
       pub entry_count: usize,
       pub total_words: usize,
       pub mood_summary: Option<String>,
       pub topics: Vec<String>,
   }
   ```

2. Implement `From<&JournalSession> for SessionFrontmatter`
3. Refactor `save_session()` to use structured serialization:

   ```rust
   let frontmatter = SessionFrontmatter::from(&session);
   let patch = Patch {
       frontmatter: Some(serde_json::to_value(frontmatter)?),
       body: Some(serde_json::to_string(&session.transcript)?),
       // ...
   };
   ```

4. Remove manual `json!` macro usage
5. Update session loading to use new structure

### Testing Strategy

- Verify session data round-trips correctly
- Test backwards compatibility with existing sessions
- Ensure all session fields are preserved
- Verify no data loss during conversion

