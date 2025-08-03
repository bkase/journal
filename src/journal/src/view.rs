use crate::state::{JournalSession, Speaker, State, WriteResult};

/// Main view function that renders the current state
pub fn view(state: &State) {
    match state {
        State::Initializing => render_initializing(),
        State::PromptingForNew => render_prompting_for_new(),
        State::InSession(session) => render_in_session(session),
        State::Analyzing(session) => render_analyzing(session),
        State::AnalysisReady {
            session: _,
            analysis,
        } => render_analysis_ready(analysis),
        State::Done(result) => render_done(result),
        State::Error(error) => render_error(&error.to_string()),
    }
}

/// Render the initializing state
fn render_initializing() {
    // Minimal output during initialization
}

/// Render the mode selection prompt
fn render_prompting_for_new() {
    println!("\n🌅 Welcome to your journal!");
    println!("What kind of session would you like to start?");
    println!("  (m)orning - Start your day with intention");
    println!("  (e)vening - Reflect on your day");
    print!("\nChoice (m/e): ");
    use std::io::{self, Write};
    io::stdout().flush().unwrap();
}

/// Render the in-session state with the current transcript
fn render_in_session(session: &JournalSession) {
    // Show the latest entry in the transcript
    if let Some(latest) = session.transcript.last() {
        match &latest.speaker {
            Speaker::Coach => {
                // For coach messages, check if it's a question or response
                if latest.content.ends_with('?') {
                    println!("\n💭 {}", latest.content);
                    print!("\n> ");
                    use std::io::{self, Write};
                    io::stdout().flush().unwrap();
                } else {
                    println!("\n🧘 Coach: {}", latest.content);
                    println!("\n⏸️  Press (s)top to end session or continue sharing...");
                }
            }
            Speaker::User => {
                // User input is already shown when typed, no need to re-display
            }
            Speaker::System => {
                println!("\n✨ {}", latest.content);
            }
        }
    }
}

/// Render the analyzing state
fn render_analyzing(_session: &JournalSession) {
    println!("\n🔍 Analyzing your session...");
}

/// Render the analysis ready state
fn render_analysis_ready(analysis: &str) {
    println!("\n🧠 **AI Analysis of Your Session**");
    println!("{}", "=".repeat(50));
    println!("{analysis}");
    println!("{}", "=".repeat(50));
}

/// Render the completion state
fn render_done(result: &WriteResult) {
    println!("\n✨ **Session Complete!**");
    println!(
        "📝 Your journal entry has been saved to: {}",
        result.entry_path
    );
    println!("🔍 The AI analysis has been included in your entry for future reference.");
}

/// Render error messages
fn render_error(msg: &str) {
    eprintln!("\n❌ Error: {msg}");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::{SessionMetadata, SessionMode, TranscriptEntry};
    use chrono::Utc;
    use uuid::Uuid;

    #[test]
    fn test_view_renders_all_states() {
        // Test that view doesn't panic for any state

        // Initializing
        view(&State::Initializing);

        // PromptingForNew
        view(&State::PromptingForNew);

        // InSession
        let session = JournalSession {
            mode: SessionMode::Morning,
            transcript: vec![TranscriptEntry {
                timestamp: Utc::now(),
                speaker: Speaker::Coach,
                content: "What are your intentions for today?".to_string(),
            }],
            metadata: SessionMetadata {
                session_doc_id: Some(Uuid::new_v4()),
                final_entry_id: None,
                completed_at: None,
                custom_fields: std::collections::HashMap::new(),
            },
        };
        view(&State::InSession(session.clone()));

        // Analyzing
        view(&State::Analyzing(session.clone()));

        // AnalysisReady
        view(&State::AnalysisReady {
            session,
            analysis: "Great session!".to_string(),
        });

        // Done
        view(&State::Done(WriteResult {
            entry_id: Uuid::new_v4(),
            entry_path: "/tmp/test/entry.md".to_string(),
            session_completed: true,
        }));

        // Error
        view(&State::Error(crate::error::Error::System("Test error".to_string())));
    }

    #[test]
    fn test_render_in_session_handles_different_speakers() {
        let mut session = JournalSession {
            mode: SessionMode::Evening,
            transcript: vec![],
            metadata: SessionMetadata {
                session_doc_id: Some(Uuid::new_v4()),
                final_entry_id: None,
                completed_at: None,
                custom_fields: std::collections::HashMap::new(),
            },
        };

        // Test coach question
        session.transcript.push(TranscriptEntry {
            timestamp: Utc::now(),
            speaker: Speaker::Coach,
            content: "How was your day?".to_string(),
        });
        render_in_session(&session);

        // Test coach response (non-question)
        session.transcript.push(TranscriptEntry {
            timestamp: Utc::now(),
            speaker: Speaker::Coach,
            content: "That sounds wonderful.".to_string(),
        });
        render_in_session(&session);

        // Test system message
        session.transcript.push(TranscriptEntry {
            timestamp: Utc::now(),
            speaker: Speaker::System,
            content: "Session saved.".to_string(),
        });
        render_in_session(&session);
    }
}
