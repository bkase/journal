use crate::state::{JournalSession, Speaker};
use aethel_core::{apply_patch, read_doc, Patch, PatchMode};
use anyhow::{Context, Result};
use chrono::Utc;
use serde_json::{json, Value};
use std::path::{Path, PathBuf};
use std::process::Command;
use tokio::fs;
use uuid::Uuid;

#[derive(Debug, Clone)]
#[allow(dead_code)] // Some variants are for future use
pub enum Effect {
    ShowModePrompt,
    ShowQuestion(String),
    ShowCoachResponse(String),
    ShowSummary(String),
    ShowMessage(String),
    ShowError(String),
    PromptForUserInput,
    SaveSession(JournalSession),
    LoadSession(Uuid),
    UpdateIndex(Uuid),
    ClearIndex,
    RequestCoachResponse {
        session: JournalSession,
        user_response: String,
    },
    CreateFinalEntry {
        session: JournalSession,
        entry_id: Uuid,
    },
    InitializeVault(PathBuf),
    InstallJournalPack(PathBuf),
}

pub struct EffectRunner {
    pub vault_path: PathBuf,
}

impl EffectRunner {
    pub fn new(vault_path: PathBuf) -> Self {
        Self { vault_path }
    }

    pub async fn run_effect(&self, effect: Effect) -> Result<Option<crate::action::Action>> {
        match effect {
            Effect::ShowModePrompt => {
                self.show_mode_prompt().await;
                Ok(None)
            }
            Effect::ShowQuestion(question) => {
                self.show_question(&question).await;
                Ok(None)
            }
            Effect::ShowCoachResponse(response) => {
                self.show_coach_response(&response).await;
                Ok(None)
            }
            Effect::ShowSummary(summary) => {
                self.show_summary(&summary).await;
                Ok(None)
            }
            Effect::ShowMessage(message) => {
                self.show_message(&message).await;
                Ok(None)
            }
            Effect::ShowError(error) => {
                self.show_error(&error).await;
                Ok(None)
            }
            Effect::PromptForUserInput => {
                self.prompt_for_user_input().await;
                Ok(None)
            }
            Effect::SaveSession(session) => {
                self.save_session(&session).await?;
                Ok(None)
            }
            Effect::LoadSession(session_id) => {
                let _session = self.load_session(session_id).await?;
                // Return action to transition to InSession state
                Ok(Some(crate::action::Action::UserResponse(
                    "session_loaded".to_string(),
                )))
            }
            Effect::UpdateIndex(session_id) => {
                self.update_index(session_id).await?;
                Ok(None)
            }
            Effect::ClearIndex => {
                self.clear_index().await?;
                Ok(None)
            }
            Effect::RequestCoachResponse {
                session,
                user_response,
            } => {
                let response = self
                    .request_coach_response(&session, &user_response)
                    .await?;
                Ok(Some(crate::action::Action::CoachResponse(response)))
            }
            Effect::CreateFinalEntry { session, entry_id } => {
                self.create_final_entry(&session, entry_id).await?;
                Ok(Some(crate::action::Action::Complete))
            }
            Effect::InitializeVault(path) => {
                self.initialize_vault(&path).await?;
                Ok(None)
            }
            Effect::InstallJournalPack(path) => {
                self.install_journal_pack(&path).await?;
                Ok(None)
            }
        }
    }

    async fn show_mode_prompt(&self) {
        println!("\n🌅 Welcome to your journal!");
        println!("What kind of session would you like to start?");
        println!("  (m)orning - Start your day with intention");
        println!("  (e)vening - Reflect on your day");
        print!("\nChoice (m/e): ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
    }

    async fn show_question(&self, question: &str) {
        println!("\n💭 {question}");
        print!("\n> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
    }

    async fn show_coach_response(&self, response: &str) {
        println!("\n🧘 Coach: {response}");
    }

    async fn show_summary(&self, summary: &str) {
        println!("\n📖 Session Summary:");
        println!("{summary}");
    }

    async fn show_message(&self, message: &str) {
        println!("\n✨ {message}");
    }

    async fn show_error(&self, error: &str) {
        eprintln!("\n❌ Error: {error}");
    }

    async fn prompt_for_user_input(&self) {
        print!("\n> ");
        use std::io::{self, Write};
        io::stdout().flush().unwrap();
    }

    fn ensure_vault_exists(&self) -> Result<()> {
        // Check if vault exists, if not initialize it
        if !self.vault_path.join(".aethel").exists() {
            // Create basic vault structure
            std::fs::create_dir_all(self.vault_path.join("docs"))
                .context("Failed to create docs directory")?;
            std::fs::create_dir_all(self.vault_path.join("packs"))
                .context("Failed to create packs directory")?;
            std::fs::create_dir_all(self.vault_path.join(".aethel"))
                .context("Failed to create .aethel directory")?;
        }
        Ok(())
    }

    async fn save_session(&self, session: &JournalSession) -> Result<()> {
        self.ensure_vault_exists()?;

        let uuid = session.metadata.session_doc_id.unwrap_or_else(Uuid::new_v4);

        let frontmatter = json!({
            "id": session.id,
            "mode": session.mode,
            "created_at": session.created_at,
            "updated_at": session.updated_at,
            "metadata": session.metadata
        });

        let body = serde_json::to_string_pretty(&session.transcript)
            .context("Failed to serialize transcript")?;

        let patch = Patch {
            uuid: if session.metadata.session_doc_id.is_some() {
                Some(uuid)
            } else {
                None
            },
            doc_type: Some("journal.session".to_string()),
            mode: if session.metadata.session_doc_id.is_some() {
                PatchMode::MergeFrontmatter
            } else {
                PatchMode::Create
            },
            frontmatter: Some(frontmatter),
            body: Some(body),
        };

        apply_patch(&self.vault_path, patch).context("Failed to save session document")?;

        Ok(())
    }

    async fn load_session(&self, session_id: Uuid) -> Result<JournalSession> {
        self.ensure_vault_exists()?;

        // Try to read the session document by UUID
        let doc =
            read_doc(&self.vault_path, &session_id).context("Failed to load session document")?;

        // Parse the transcript from the body
        let transcript =
            serde_json::from_str(&doc.body).context("Failed to parse session transcript")?;

        // Extract session data from frontmatter
        let session_data = &doc.frontmatter_extra;
        let session = JournalSession {
            id: session_data
                .get("id")
                .and_then(|v| v.as_str())
                .and_then(|s| Uuid::parse_str(s).ok())
                .unwrap_or(session_id),
            mode: serde_json::from_value(
                session_data
                    .get("mode")
                    .cloned()
                    .unwrap_or(json!("Morning")),
            )
            .unwrap_or(crate::state::SessionMode::Morning),
            transcript,
            metadata: serde_json::from_value(
                session_data.get("metadata").cloned().unwrap_or(json!({})),
            )
            .context("Failed to parse session metadata")?,
            created_at: session_data
                .get("created_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
            updated_at: session_data
                .get("updated_at")
                .and_then(|v| v.as_str())
                .and_then(|s| chrono::DateTime::parse_from_rfc3339(s).ok())
                .map(|dt| dt.with_timezone(&chrono::Utc))
                .unwrap_or_else(chrono::Utc::now),
        };

        Ok(session)
    }

    async fn update_index(&self, session_id: Uuid) -> Result<()> {
        let index_path = self.vault_path.join(".aethel/indexes/journal.index.json");

        // Ensure the directory exists
        if let Some(parent) = index_path.parent() {
            fs::create_dir_all(parent)
                .await
                .context("Failed to create indexes directory")?;
        }

        let index_data = json!({
            "active_session": session_id,
            "updated_at": Utc::now()
        });

        fs::write(&index_path, serde_json::to_string_pretty(&index_data)?)
            .await
            .context("Failed to write index file")?;

        Ok(())
    }

    async fn clear_index(&self) -> Result<()> {
        let index_path = self.vault_path.join(".aethel/indexes/journal.index.json");

        if index_path.exists() {
            fs::remove_file(&index_path)
                .await
                .context("Failed to remove index file")?;
        }

        Ok(())
    }

    async fn request_coach_response(
        &self,
        session: &JournalSession,
        user_response: &str,
    ) -> Result<String> {
        let context = session.mode.get_coaching_context();
        let conversation_history = session
            .transcript
            .iter()
            .filter(|entry| matches!(entry.speaker, Speaker::User | Speaker::Coach))
            .map(|entry| {
                let role = match entry.speaker {
                    Speaker::User => "user",
                    Speaker::Coach => "assistant",
                    _ => "system",
                };
                format!("{}: {}", role, entry.content)
            })
            .collect::<Vec<_>>()
            .join("\n");

        let prompt = format!(
            "{context}\n\nConversation so far:\n{conversation_history}\n\nLatest user response: {user_response}\n\nPlease respond as an empathetic coach with a follow-up question or reflection that helps deepen their self-awareness."
        );

        // Call claude CLI as subprocess
        let output = Command::new("claude")
            .arg("-p")
            .arg(&prompt)
            .output()
            .context("Failed to execute claude command")?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("Claude command failed: {}", stderr);
        }

        let response = String::from_utf8(output.stdout)
            .context("Invalid UTF-8 in claude response")?
            .trim()
            .to_string();

        Ok(response)
    }

    async fn create_final_entry(&self, session: &JournalSession, entry_id: Uuid) -> Result<()> {
        self.ensure_vault_exists()?;

        let frontmatter = json!({
            "id": entry_id,
            "session_id": session.id,
            "mode": session.mode,
            "tags": [],
            "mood": extract_mood_from_session(session),
            "energy": extract_energy_from_session(session)
        });

        let body = format!(
            "# {} Journal Entry\n\n{}",
            match session.mode {
                crate::state::SessionMode::Morning => "Morning",
                crate::state::SessionMode::Evening => "Evening",
            },
            session.get_conversation_summary()
        );

        let patch = Patch {
            uuid: None, // Create mode requires uuid to be None
            doc_type: Some("journal.entry".to_string()),
            mode: PatchMode::Create,
            frontmatter: Some(frontmatter),
            body: Some(body),
        };

        apply_patch(&self.vault_path, patch).context("Failed to create final journal entry")?;

        Ok(())
    }

    async fn initialize_vault(&self, path: &Path) -> Result<()> {
        // Create vault directory structure
        std::fs::create_dir_all(path.join("docs")).context("Failed to create docs directory")?;
        std::fs::create_dir_all(path.join("packs")).context("Failed to create packs directory")?;
        std::fs::create_dir_all(path.join(".aethel"))
            .context("Failed to create .aethel directory")?;

        // Install the journal pack
        self.install_journal_pack(path).await?;

        Ok(())
    }

    async fn install_journal_pack(&self, vault_path: &Path) -> Result<()> {
        // Create the journal pack definition
        let pack_definition = create_journal_pack_definition();

        let pack_path = vault_path.join(".aethel/packs/journal@0.1.0");
        fs::create_dir_all(&pack_path)
            .await
            .context("Failed to create pack directory")?;

        let pack_file = pack_path.join("pack.json");
        fs::write(&pack_file, serde_json::to_string_pretty(&pack_definition)?)
            .await
            .context("Failed to write pack definition")?;

        Ok(())
    }
}

fn extract_mood_from_session(session: &JournalSession) -> Option<String> {
    // Simple mood extraction - look for mood-related keywords in user responses
    for entry in session.get_user_responses() {
        let content = entry.content.to_lowercase();
        if content.contains("happy") || content.contains("great") || content.contains("wonderful") {
            return Some("positive".to_string());
        } else if content.contains("sad")
            || content.contains("difficult")
            || content.contains("hard")
        {
            return Some("challenging".to_string());
        } else if content.contains("okay")
            || content.contains("fine")
            || content.contains("neutral")
        {
            return Some("neutral".to_string());
        }
    }
    None
}

fn extract_energy_from_session(session: &JournalSession) -> Option<String> {
    // Simple energy extraction - look for energy-related keywords
    for entry in session.get_user_responses() {
        let content = entry.content.to_lowercase();
        if content.contains("energetic")
            || content.contains("motivated")
            || content.contains("high")
        {
            return Some("high".to_string());
        } else if content.contains("tired")
            || content.contains("exhausted")
            || content.contains("low")
        {
            return Some("low".to_string());
        } else if content.contains("medium") || content.contains("moderate") {
            return Some("medium".to_string());
        }
    }
    None
}

fn create_journal_pack_definition() -> Value {
    json!({
        "name": "journal",
        "version": "0.1.0",
        "description": "Interactive journaling pack for empathetic self-reflection",
        "types": {
            "journal.session": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "format": "uuid" },
                    "mode": { "enum": ["Morning", "Evening"] },
                    "created_at": { "type": "string", "format": "date-time" },
                    "updated_at": { "type": "string", "format": "date-time" },
                    "metadata": { "type": "object" }
                },
                "required": ["id", "mode", "created_at", "updated_at"]
            },
            "journal.entry": {
                "type": "object",
                "properties": {
                    "id": { "type": "string", "format": "uuid" },
                    "session_id": { "type": "string", "format": "uuid" },
                    "mode": { "enum": ["Morning", "Evening"] },
                    "created_at": { "type": "string", "format": "date-time" },
                    "tags": {
                        "type": "array",
                        "items": { "type": "string" }
                    },
                    "mood": { "type": "string" },
                    "energy": { "type": "string" }
                },
                "required": ["id", "session_id", "mode", "created_at"]
            }
        },
        "templates": {
            "journal.morning": {
                "type": "journal.entry",
                "frontmatter": {
                    "type": "journal.entry",
                    "mode": "Morning",
                    "tags": ["morning", "reflection"],
                    "mood": null,
                    "energy": null
                },
                "body": "# Morning Journal Entry\n\n## How I'm feeling:\n\n## My intentions for today:\n\n## What I'm grateful for:\n\n"
            },
            "journal.evening": {
                "type": "journal.entry",
                "frontmatter": {
                    "type": "journal.entry",
                    "mode": "Evening",
                    "tags": ["evening", "reflection"],
                    "mood": null,
                    "energy": null
                },
                "body": "# Evening Journal Entry\n\n## How my day went:\n\n## What I learned:\n\n## What I'm grateful for:\n\n## Tomorrow I will:\n\n"
            }
        }
    })
}
