# Todos

## 3. Clean Data Structures

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

