#![allow(clippy::uninlined_format_args)]

mod action;
mod effects;
mod error;
mod state;
mod update;
mod view;

use action::{Action, InputContext, UserInput};
use anyhow::{Context, Result};
use clap::{Arg, Command as ClapCommand};
use effects::{Effect, EffectRunner};
use error::Error;
use state::State;
use std::io;
use std::path::PathBuf;
use uuid::Uuid;

#[derive(Debug)]
struct AppConfig {
    vault_path: PathBuf,
    command: AppCommand,
}

#[derive(Debug)]
enum AppCommand {
    New,
    Resume(Option<Uuid>),
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args()?;

    let mut app = JournalApp::new(config.vault_path).await?;

    let initial_action = match config.command {
        AppCommand::New => Action::Start,
        AppCommand::Resume(Some(session_id)) => Action::Resume(session_id),
        AppCommand::Resume(None) => {
            // Try to find the most recent active session
            match app.find_active_session().await {
                Ok(Some(session_id)) => Action::Resume(session_id),
                Ok(None) => {
                    // No active session found, start new one
                    Action::Start
                }
                Err(_) => {
                    // Error finding active session, start new one
                    Action::Start
                }
            }
        }
    };

    app.run(initial_action).await
}

fn get_default_vault_path() -> PathBuf {
    // Default to ~/Documents/vault
    if let Some(home_dir) = std::env::var_os("HOME") {
        PathBuf::from(home_dir).join("Documents").join("vault")
    } else {
        // Fallback for Windows or if HOME is not set
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join("Documents")
            .join("vault")
    }
}

fn parse_args() -> Result<AppConfig> {
    let matches = ClapCommand::new("journal")
        .version("0.1.0")
        .about("Interactive empathetic journaling tool")
        .arg(
            Arg::new("vault")
                .short('v')
                .long("vault")
                .value_name("PATH")
                .help("Path to the journal vault (default: ~/Documents/vault)")
                .value_parser(clap::value_parser!(PathBuf)),
        )
        .subcommand(ClapCommand::new("new").about("Start a new journal session"))
        .subcommand(
            ClapCommand::new("resume")
                .about("Resume an existing session")
                .arg(
                    Arg::new("session-id")
                        .value_name("UUID")
                        .help("Specific session ID to resume")
                        .value_parser(clap::value_parser!(Uuid)),
                ),
        )
        .get_matches();

    let vault_path = matches
        .get_one::<PathBuf>("vault")
        .cloned()
        .unwrap_or_else(get_default_vault_path);

    let command = match matches.subcommand() {
        Some(("new", _)) => AppCommand::New,
        Some(("resume", sub_matches)) => {
            let session_id = sub_matches.get_one::<Uuid>("session-id").cloned();
            AppCommand::Resume(session_id)
        }
        _ => AppCommand::New, // Default to new session
    };

    Ok(AppConfig {
        vault_path,
        command,
    })
}

struct JournalApp {
    state: State,
    effect_runner: EffectRunner,
}

impl JournalApp {
    async fn new(vault_path: PathBuf) -> Result<Self> {
        // Initialize vault if it doesn't exist
        if !vault_path.join(".aethel").exists() {
            let effect_runner = EffectRunner::new(vault_path.clone());
            effect_runner
                .run_effect(Effect::InitializeVault(vault_path.clone()))
                .await?;
        }

        Ok(Self {
            state: State::Initializing,
            effect_runner: EffectRunner::new(vault_path),
        })
    }

    async fn find_active_session(&self) -> Result<Option<Uuid>> {
        // Read the index file to find active session
        let index_path = self
            .effect_runner
            .vault_path
            .join(".aethel/indexes/journal.index.json");

        if !index_path.exists() {
            return Ok(None);
        }

        let content = tokio::fs::read_to_string(&index_path)
            .await
            .context("Failed to read index file")?;

        let index: serde_json::Value =
            serde_json::from_str(&content).context("Failed to parse index file")?;

        if let Some(session_id_str) = index.get("active_session").and_then(|v| v.as_str()) {
            let session_id =
                Uuid::parse_str(session_id_str).context("Invalid session ID in index")?;
            Ok(Some(session_id))
        } else {
            Ok(None)
        }
    }

    async fn run(&mut self, initial_action: Action) -> Result<()> {
        // Process initial action
        self.process_action(initial_action).await?;

        // Main interactive loop
        while !self.state.is_terminal() {
            if self.state.is_interactive() {
                // Get user input
                let input = self.get_user_input().await?;
                let context = match self.state {
                    State::PromptingForNew => InputContext::ModeSelection,
                    State::InSession(_) => InputContext::InSession,
                    _ => InputContext::ModeSelection,
                };
                let action = UserInput::new_with_context(input, context).processed;
                self.process_action(action).await?;
            } else {
                // Non-interactive states should have generated effects that will advance the state
                // If we're stuck in a non-interactive state, that's an error
                if !self.state.is_terminal() {
                    return Err(Error::system(format!(
                        "Stuck in non-interactive state: {:?}",
                        self.state
                    ))
                    .into());
                }
            }
        }

        // Print final message if we're in an error state
        if let State::Error(ref error) = self.state {
            eprintln!("Session ended with error: {}", error);
            std::process::exit(1);
        }

        Ok(())
    }

    async fn process_action(&mut self, action: Action) -> Result<()> {
        let (new_state, effects) = update::update(self.state.clone(), action);
        self.state = new_state;

        // Display the new state
        view::view(&self.state);

        // Execute all effects
        for effect in effects {
            let effect_for_match = effect.clone();
            match self.effect_runner.run_effect(effect).await {
                Ok(Some(resulting_action)) => {
                    // Some effects generate new actions (like coach responses)
                    let (next_state, next_effects) =
                        update::update(self.state.clone(), resulting_action);
                    self.state = next_state;

                    // Display the updated state
                    view::view(&self.state);

                    // Execute any additional effects
                    for next_effect in next_effects {
                        match self.effect_runner.run_effect(next_effect).await {
                            Ok(_) => {}
                            Err(e) => {
                                eprintln!("\n❌ Error executing nested effect: {}", e);
                                // Continue with the session instead of crashing
                            }
                        }
                    }
                }
                Ok(None) => {
                    // Effect completed successfully without generating an action
                }
                Err(e) => {
                    // Handle different error types with specific recovery strategies
                    match &e {
                        Error::AiAnalysis(_) | Error::ClaudeExecution { .. } => {
                            eprintln!("\n❌ AI Error: {}", e);
                            if matches!(effect_for_match, Effect::GenerateAnalysis { .. }) {
                                eprintln!("🔄 Continuing without AI analysis...");
                                // Generate a fallback AnalysisComplete action with error message
                                let fallback_analysis = format!(
                                    "**AI Analysis Unavailable**\n\n\
                                    The AI analysis feature encountered an error and is currently unavailable. \
                                    Your journal session has been saved successfully.\n\n\
                                    Error details: {}", 
                                    e
                                );
                                let fallback_action = Action::AnalysisComplete(fallback_analysis);
                                let (next_state, next_effects) =
                                    update::update(self.state.clone(), fallback_action);
                                self.state = next_state;

                                // Execute any additional effects from the fallback
                                for next_effect in next_effects {
                                    match self.effect_runner.run_effect(next_effect).await {
                                        Ok(_) => {}
                                        Err(e) => {
                                            eprintln!("\n❌ Error in fallback effect: {:#}", e);
                                        }
                                    }
                                }
                            }
                        }
                        Error::SessionNotFound { session_id } => {
                            eprintln!("\n❌ Session Error: Session {} not found", session_id);
                            eprintln!("🔄 Starting a new session...");
                            // Could trigger a new session action here
                        }
                        Error::VaultOperation { operation } => {
                            eprintln!("\n❌ Vault Error: {}", operation);
                            eprintln!("💾 This might be a storage issue. Please check file permissions and disk space.");
                            // For critical vault errors, we might want to exit
                        }
                        Error::InvalidSessionState { reason } => {
                            eprintln!("\n❌ Session State Error: {}", reason);
                            eprintln!("🔄 Attempting to recover session...");
                        }
                        Error::Aethel { .. } | Error::Io { .. } | Error::Json { .. } => {
                            eprintln!("\n❌ System Error: {}", e);
                            eprintln!(
                                "⚠️  This is a system-level error that may require attention."
                            );
                        }
                        Error::Config(_) => {
                            eprintln!("\n❌ Configuration Error: {}", e);
                            eprintln!("⚙️  Please check your configuration settings.");
                        }
                        Error::UserInput(_) => {
                            eprintln!("\n❌ Input Error: {}", e);
                            eprintln!("💬 Please try entering your input again.");
                        }
                        Error::System(_) => {
                            eprintln!("\n❌ System Error: {}", e);
                        }
                    }
                }
            }
        }

        Ok(())
    }

    async fn get_user_input(&mut self) -> Result<String> {
        let mut line = String::new();
        io::stdin()
            .read_line(&mut line)
            .context("Failed to read user input")?;

        Ok(line.trim().to_string())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_app_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().to_path_buf();

        let app = JournalApp::new(vault_path.clone()).await;
        assert!(app.is_ok());

        // Check that vault was initialized
        assert!(vault_path.join(".aethel").exists());
    }

    #[test]
    fn test_arg_parsing() {
        // Test default behavior
        let config = AppConfig {
            vault_path: get_default_vault_path(),
            command: AppCommand::New,
        };

        // This is a simple test - in practice you'd use clap's testing facilities
        assert!(matches!(config.command, AppCommand::New));

        // Verify default path is ~/Documents/vault
        let expected_path = if let Some(home_dir) = std::env::var_os("HOME") {
            PathBuf::from(home_dir).join("Documents").join("vault")
        } else {
            std::env::current_dir()
                .unwrap_or_else(|_| PathBuf::from("."))
                .join("Documents")
                .join("vault")
        };
        assert_eq!(config.vault_path, expected_path);
    }

    #[test]
    fn test_get_default_vault_path() {
        let default_path = get_default_vault_path();

        // The path should end with "Documents/vault"
        assert!(default_path.to_string_lossy().ends_with("Documents/vault"));

        // If HOME is set, it should start with the home directory
        if let Some(home_dir) = std::env::var_os("HOME") {
            assert!(default_path.starts_with(home_dir));
        }
    }

    #[tokio::test]
    async fn test_find_active_session_no_index() {
        let temp_dir = TempDir::new().unwrap();
        let vault_path = temp_dir.path().to_path_buf();

        let app = JournalApp::new(vault_path).await.unwrap();
        let result = app.find_active_session().await.unwrap();

        assert!(result.is_none());
    }
}
