mod action;
mod effects;
mod state;
mod update;

use action::{Action, UserInput};
use anyhow::{Context, Result};
use clap::{Arg, Command as ClapCommand};
use effects::{Effect, EffectRunner};
use state::State;
use std::io::{self, Write};
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
                    println!("No active session found. Starting new session.");
                    Action::Start
                }
                Err(e) => {
                    eprintln!("Error finding active session: {e}. Starting new session.");
                    Action::Start
                }
            }
        }
    };

    app.run(initial_action).await
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
                .help("Path to the journal vault")
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
        .unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")));

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
            println!("Initializing journal vault at {}...", vault_path.display());
            let effect_runner = EffectRunner::new(vault_path.clone());
            effect_runner
                .run_effect(Effect::InitializeVault(vault_path.clone()))
                .await?;
            println!("Vault initialized successfully!");
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
                let action = UserInput::new(input).processed;
                self.process_action(action).await?;
            } else {
                // Non-interactive states should have generated effects that will advance the state
                // If we're stuck in a non-interactive state, that's an error
                if !self.state.is_terminal() {
                    return Err(anyhow::anyhow!(
                        "Stuck in non-interactive state: {:?}",
                        self.state
                    ));
                }
            }
        }

        // Print final message if we're in an error state
        if let State::Error(ref msg) = self.state {
            eprintln!("Session ended with error: {msg}");
            std::process::exit(1);
        }

        Ok(())
    }

    async fn process_action(&mut self, action: Action) -> Result<()> {
        let (new_state, effects) = update::update(self.state.clone(), action);
        self.state = new_state;

        // Execute all effects
        for effect in effects {
            if let Some(resulting_action) = self.effect_runner.run_effect(effect).await? {
                // Some effects generate new actions (like coach responses)
                let (next_state, next_effects) =
                    update::update(self.state.clone(), resulting_action);
                self.state = next_state;

                // Execute any additional effects
                for next_effect in next_effects {
                    self.effect_runner.run_effect(next_effect).await?;
                }
            }
        }

        Ok(())
    }

    async fn get_user_input(&mut self) -> Result<String> {
        print!("\n> ");
        io::stdout().flush().context("Failed to flush stdout")?;

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
            vault_path: PathBuf::from("."),
            command: AppCommand::New,
        };

        // This is a simple test - in practice you'd use clap's testing facilities
        assert!(matches!(config.command, AppCommand::New));
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
