Of course. Moving the workflow loop into a single, stateful Rust application is a much more robust and user-friendly design. It eliminates the need for an external orchestrator and provides a seamless, interactive experience, while still leveraging the Aethel vault as the single source of truth for session state.

This revised architecture aligns perfectly with the Elm-like pattern seen in the `Momentum` project, where an application loop handles user input and side effects internally.

Here is the detailed Software Design Document for this improved, interactive CLI tool.

---

# **Software Design Document: `journal` (Interactive)**

*   **Version:** 2.0
*   **Date:** 2025-08-02
*   **Status:** Final

## 1. Introduction & Guiding Philosophy

This document details the software design for `journal`, a stateful, interactive command-line tool for conducting empathetic journaling sessions. This version revises the previous stateless design, encapsulating the entire user workflow within a single Rust application run.

The core philosophy remains: the **Aethel vault is the single source of truth**. The application will load its state from the vault on startup, persist changes during the session, and allow users to seamlessly resume interrupted sessions. We will adopt the Elm-like architecture (`State`, `Action`, `update`, `effects`) from the `Momentum` project to manage the interactive loop, ensuring a clean separation of concerns and high testability.

## 2. High-Level Architecture

The application will be a single, long-running process for the duration of a journaling session. It reads state from and writes state to the Aethel vault on each conversational turn.

```mermaid
graph TD
    subgraph L1: journal
        A[main()] --> B{Startup Logic};
        B -- Found Session --> C[Main Loop];
        B -- No Session --> D[Prompt for New Session];
        D --> C;
        C -- User Input --> E[Action];
        E --> F[update(state, action)];
        F -- New State --> C;
        F -- Effect --> G[effects::execute];
    end

    subgraph L0: Core Services
        H[Aethel Vault (via aethel-core)];
        I[Claude CLI (via Subprocess)];
    end

    G --> H;
    G --> I;
```

**Workflow:**
1.  **Startup:** The application checks the Aethel vault for an active `journal.session` document.
2.  **Resume or Start:** If an active session is found, it's loaded. If not, the user is prompted to start a `morning` or `evening` session.
3.  **Main Loop:** The application enters the `CONVERSE` stage, reading user input, getting AI coaching replies, and saving the updated transcript to the vault *after every turn*.
4.  **Finalization:** When the user ends the session, the application enters the `ANALYZE` and `SAVE` stages, creating a permanent `journal.entry` and archiving the session document.

## 3. Aethel Integration

Direct integration with `aethel-core` is central to this design.

### 3.1. The `journal` Pack

The pack will define two document types: one for the final entry and one for the transient session state.

*   **`journal.entry`:** The permanent, immutable record of a completed session. (Schema remains as previously defined).
*   **`journal.session`:** A mutable, transient document that holds the state of an in-progress session. It is created on `start` and `archived` on `end`.

**Schema (`packs/journal/types/session.json`):**
```json
{
  "$schema": "http://json-schema.org/draft-07/schema#",
  "title": "Journal Session State",
  "type": "object",
  "properties": {
    "session_type": { "type": "string", "enum": ["morning", "evening"] },
    "start_time": { "type": "string", "format": "date-time" }
  },
  "required": ["session_type", "start_time"]
}
```
The body of the `journal.session` document will contain the live, in-progress transcript.

### 3.2. Active Session Discovery (Index File)

To avoid scanning the entire vault for an active session, we will adopt the robust indexing pattern from the `Momentum` project's `todos`.

*   **Index File:** A dedicated index file will live at `<vault>/.aethel/indexes/journal.index.json`.
*   **Content:**
    ```json
    {
      "active_session_uuid": "f81d4fae-7dec-11d0-a765-00a0c91e6bf6"
    }
    ```
    If no session is active, the value will be `null`.
*   **Operations:**
    *   **On Start:** The new `journal.session` UUID is written to this file.
    *   **On End:** The `active_session_uuid` key is set to `null`.
    *   **On Startup:** The application reads this file first to find the active session UUID.

## 4. CLI Design

The user interface is simplified to a single command.

```bash
journal 1.0
An empathetic journaling tool with Aethel vault integration

USAGE:
    journal [OPTIONS]

OPTIONS:
    --vault <PATH>    Path to the Aethel vault (overrides JOURNAL_VAULT_PATH env var)
```
The application will determine whether to start or resume a session based on the state of the vault.

## 5. Core Workflow Implementation (The Elm Architecture)

### 5.1. `State` Enum (`state.rs`)

This enum will represent the different modes of the interactive application.

```rust
// In src/state.rs
pub enum State {
    Initializing,
    PromptingForNew, // Asking user "morning or evening?"
    InSession(JournalSession),
    Analyzing,
    Saving,
    Done(WriteResult),
    Error(String),
}

pub struct JournalSession {
    pub uuid: Uuid,
    pub session_type: SessionType,
    pub start_time: DateTime<Utc>,
    pub transcript: String, // The full transcript, loaded from the Doc body
}
```

### 5.2. `Action` Enum (`action.rs`)

This enum represents all events that can drive state changes.

```rust
// In src/action.rs
pub enum Action {
    // Startup
    Initialized(Option<JournalSession>), // Carries a session if one was found
    StartNew(SessionType),

    // Conversation Loop
    UserInput(String),
    ReceiveCoachReply(String),
    EndSession,

    // Finalization
    AnalysisComplete(AnalysisResult),
    SaveComplete(WriteResult),

    // General
    FatalError(String),
}
```

### 5.3. `update` Function (`update.rs`)

This pure function is the core of the application's logic.

```rust
// In src/update.rs
pub fn update(state: State, action: Action, _env: &Environment) -> (State, Option<Effect>) {
    match (state, action) {
        (State::Initializing, Action::Initialized(Some(session))) => {
            // Found an active session, jump right in
            (State::InSession(session), Some(Effect::PrintToConsole(
                "Resuming previous session...".to_string()
            )))
        }
        (State::Initializing, Action::Initialized(None)) => {
            // No active session, prompt the user
            (State::PromptingForNew, None)
        }
        (State::PromptingForNew, Action::StartNew(session_type)) => {
            // User chose a session type, create the session doc
            let effect = Effect::CreateSessionDoc(session_type);
            (State::Initializing, Some(effect)) // Go back to initializing to load the new doc
        }
        (State::InSession(mut session), Action::UserInput(input)) => {
            // Append user input to transcript and get a coach reply
            session.transcript.push_str(&format!("\n\nUser: {}", input));
            let effect = Effect::GetCoachReply(session.clone());
            (State::InSession(session), Some(effect))
        }
        (State::InSession(mut session), Action::ReceiveCoachReply(reply)) => {
            // Append coach reply and save the updated session
            session.transcript.push_str(&format!("\n\nCoach: {}", reply));
            let effect = Effect::Composite(vec![
                Effect::UpdateSessionDoc(session.clone()),
                Effect::PrintToConsole(format!("\nCoach: {}\n", reply)),
            ]);
            (State::InSession(session), Some(effect))
        }
        (State::InSession(session), Action::EndSession) => {
            // User wants to end, start the analysis
            (State::Analyzing, Some(Effect::AnalyzeTranscript(session)))
        }
        (_, Action::AnalysisComplete(analysis)) => {
            // Analysis is done, now save the final entry
            (State::Saving, Some(Effect::SaveEntryAndArchiveSession(analysis)))
        }
        (_, Action::SaveComplete(write_result)) => {
            // All done, show success message and exit
            (State::Done(write_result), None)
        }
        // ... other state transitions and error handling
        (_, Action::FatalError(msg)) => (State::Error(msg), None),
    }
}
```

### 5.4. `Effect` Enum and Handler (`effects.rs`)

This module defines and executes all side effects.

```rust
// In src/effects.rs
pub enum Effect {
    FindActiveSession,
    CreateSessionDoc(SessionType),
    UpdateSessionDoc(JournalSession),
    GetCoachReply(JournalSession),
    AnalyzeTranscript(JournalSession),
    SaveEntryAndArchiveSession(AnalysisResult),
    PrintToConsole(String),
    Composite(Vec<Effect>),
}

pub async fn execute(effect: Effect, env: &Environment) -> Result<Action> {
    match effect {
        Effect::FindActiveSession => {
            // 1. Read .aethel/indexes/journal.index.json
            // 2. If UUID found, read the journal.session Doc
            // 3. Return Action::Initialized(Some(session)) or Action::Initialized(None)
        }
        Effect::CreateSessionDoc(session_type) => {
            // 1. Create a new journal.session Doc using aethel_core::apply_patch
            // 2. Get the new UUID
            // 3. Update journal.index.json with the new UUID
            // 4. Re-run the FindActiveSession effect to load the new state
        }
        Effect::GetCoachReply(session) => {
            // 1. Call `claude -p '<coaching_prompt>'` with session.transcript
            // 2. Return Action::ReceiveCoachReply(claude_output)
        }
        // ... implementation for other effects
    }
}
```

## 6. Main Application Loop (`main.rs`)

The `main` function will orchestrate the startup and the interactive loop.

```rust
// In src/main.rs
#[tokio::main]
async fn main() -> Result<()> {
    // ... clap parsing and environment setup ...

    let mut state = State::Initializing;
    let mut effect = Some(Effect::FindActiveSession);

    loop {
        // Execute effect if one exists
        if let Some(e) = effect.take() {
            match effects::execute(e, &env).await {
                Ok(action) => {
                    (state, effect) = update::update(state, action, &env);
                }
                Err(err) => {
                    (state, effect) = update::update(state, Action::FatalError(err.to_string()), &env);
                }
            }
        }

        // Handle current state
        match &state {
            State::PromptingForNew => {
                // ... logic to ask user for "morning" or "evening" ...
                // let action = Action::StartNew(choice);
                // (state, effect) = update::update(state, action, &env);
            }
            State::InSession(_) => {
                // ... read user input from stdin ...
                // if input == "y" { action = Action::EndSession }
                // else { action = Action::UserInput(input) }
                // (state, effect) = update::update(state, action, &env);
            }
            State::Done(result) => {
                println!("✨ Journal session saved!");
                println!("   ID: {}", result.uuid);
                println!("   Path: {}", result.path);
                break; // Exit loop
            }
            State::Error(msg) => {
                eprintln!("An error occurred: {}", msg);
                break; // Exit loop
            }
            _ => { /* Initializing, Analyzing, Saving states are transient */ }
        }
    }
    Ok(())
}
```