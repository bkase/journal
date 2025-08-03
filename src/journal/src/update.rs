use crate::action::Action;
use crate::effects::Effect;
use crate::state::{JournalSession, SessionMode, Speaker, State, WriteResult};
use uuid::Uuid;

pub fn update(state: State, action: Action) -> (State, Vec<Effect>) {
    match (state, action) {
        // Starting a new journal session
        (State::Initializing, Action::Start) => {
            (State::PromptingForNew, vec![Effect::ShowModePrompt])
        }

        // Resuming an existing session
        (State::Initializing, Action::Resume(session_id)) => {
            (State::Initializing, vec![Effect::LoadSession(session_id)])
        }

        // Mode selection
        (State::PromptingForNew, Action::SelectMode(mode)) => {
            let vault_path = std::env::current_dir()
                .unwrap_or_else(|_| std::path::PathBuf::from("."))
                .to_string_lossy()
                .to_string();

            let mut session = JournalSession::new(mode, vault_path);
            let initial_questions = session.mode.get_initial_questions();

            session.add_entry(
                Speaker::System,
                format!(
                    "Starting {} journal session",
                    match mode {
                        SessionMode::Morning => "morning",
                        SessionMode::Evening => "evening",
                    }
                ),
            );

            (
                State::InSession(session.clone()),
                vec![
                    Effect::SaveSession(session.clone()),
                    Effect::UpdateIndex(session.id),
                    Effect::ShowQuestion(initial_questions[0].to_string()),
                ],
            )
        }

        // User responses during session
        (State::InSession(mut session), Action::UserResponse(response)) => {
            session.add_entry(Speaker::User, response.clone());

            let effects = vec![
                Effect::SaveSession(session.clone()),
                Effect::RequestCoachResponse {
                    session: session.clone(),
                    user_response: response,
                },
            ];

            (State::InSession(session), effects)
        }

        // Coach responses
        (State::InSession(mut session), Action::CoachResponse(response)) => {
            session.add_entry(Speaker::Coach, response);

            (
                State::InSession(session.clone()),
                vec![
                    Effect::SaveSession(session.clone()),
                    Effect::ShowCoachResponse(session.transcript.last().unwrap().content.clone()),
                    Effect::PromptForUserInput,
                ],
            )
        }

        // Moving to next question
        (State::InSession(session), Action::NextQuestion) => {
            let user_responses = session.get_user_responses().len();
            let questions = session.mode.get_initial_questions();

            if user_responses < questions.len() {
                (
                    State::InSession(session),
                    vec![Effect::ShowQuestion(questions[user_responses].to_string())],
                )
            } else {
                (State::InSession(session), vec![Effect::PromptForUserInput])
            }
        }

        // Requesting summary
        (State::InSession(session), Action::RequestSummary) => {
            let summary = session.get_conversation_summary();
            (
                State::InSession(session),
                vec![Effect::ShowSummary(summary), Effect::PromptForUserInput],
            )
        }

        // Saving session (without completing)
        (State::InSession(session), Action::Save) => (
            State::Saving,
            vec![
                Effect::SaveSession(session),
                Effect::ShowMessage("Session saved. You can resume later.".to_string()),
            ],
        ),

        // Completing session
        (State::InSession(mut session), Action::Complete) => {
            session.mark_completed();
            let entry_id = Uuid::new_v4();

            (
                State::Analyzing,
                vec![
                    Effect::SaveSession(session.clone()),
                    Effect::CreateFinalEntry {
                        session: session.clone(),
                        entry_id,
                    },
                ],
            )
        }

        // Session saved successfully (from Save action)
        (State::Saving, Action::Save) => {
            (
                State::Done(WriteResult {
                    entry_id: Uuid::new_v4(), // Temporary ID for save operations
                    entry_path: "session_saved".to_string(),
                    session_completed: false,
                }),
                vec![Effect::ClearIndex],
            )
        }

        // Final entry created successfully
        (State::Analyzing, Action::Complete) => {
            let entry_id = Uuid::new_v4();
            (
                State::Done(WriteResult {
                    entry_id,
                    entry_path: format!("entry_{entry_id}.md"),
                    session_completed: true,
                }),
                vec![
                    Effect::ClearIndex,
                    Effect::ShowMessage("Journal entry created successfully!".to_string()),
                ],
            )
        }

        // Quitting
        (_, Action::Quit) => (
            State::Done(WriteResult {
                entry_id: Uuid::new_v4(),
                entry_path: "quit".to_string(),
                session_completed: false,
            }),
            vec![Effect::ShowMessage("Goodbye!".to_string())],
        ),

        // Error handling
        (_, Action::Error(msg)) => (State::Error(msg.clone()), vec![Effect::ShowError(msg)]),

        // Session loaded successfully (from Resume)
        (State::Initializing, Action::UserResponse(_)) => {
            // This would happen after a successful session load
            // The effect handler would have loaded the session and we transition to InSession
            // This is a placeholder - the actual loaded session would be provided by the effect
            (
                State::Error("Session load not implemented yet".to_string()),
                vec![],
            )
        }

        // Invalid state transitions
        (state, action) => {
            let error_msg = format!("Invalid action {action:?} for state {state:?}");
            (
                State::Error(error_msg.clone()),
                vec![Effect::ShowError(error_msg)],
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state::SessionMode;

    #[test]
    fn test_initial_start() {
        let (new_state, effects) = update(State::Initializing, Action::Start);

        assert_eq!(new_state, State::PromptingForNew);
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], Effect::ShowModePrompt));
    }

    #[test]
    fn test_mode_selection() {
        let (new_state, effects) = update(
            State::PromptingForNew,
            Action::SelectMode(SessionMode::Morning),
        );

        assert!(matches!(new_state, State::InSession(_)));
        assert_eq!(effects.len(), 3);
        assert!(matches!(effects[1], Effect::UpdateIndex(_)));
        assert!(matches!(effects[2], Effect::ShowQuestion(_)));
    }

    #[test]
    fn test_user_response() {
        let vault_path = "/test/vault".to_string();
        let mut session = JournalSession::new(SessionMode::Morning, vault_path);
        // Add the initial system message that would be added during mode selection
        session.add_entry(
            Speaker::System,
            "Starting morning journal session".to_string(),
        );
        let initial_state = State::InSession(session);

        let (new_state, effects) = update(
            initial_state,
            Action::UserResponse("I feel great!".to_string()),
        );

        assert!(matches!(new_state, State::InSession(_)));
        assert_eq!(effects.len(), 2);

        if let State::InSession(updated_session) = new_state {
            assert_eq!(updated_session.transcript.len(), 2); // System + User
            assert_eq!(updated_session.transcript[1].content, "I feel great!");
        }
    }

    #[test]
    fn test_complete_session() {
        let vault_path = "/test/vault".to_string();
        let session = JournalSession::new(SessionMode::Morning, vault_path);
        let initial_state = State::InSession(session);

        let (new_state, effects) = update(initial_state, Action::Complete);

        assert_eq!(new_state, State::Analyzing);
        assert_eq!(effects.len(), 2);
        assert!(matches!(effects[0], Effect::SaveSession(_)));
        assert!(matches!(effects[1], Effect::CreateFinalEntry { .. }));
    }

    #[test]
    fn test_quit_action() {
        let (new_state, effects) = update(State::Initializing, Action::Quit);

        assert!(matches!(new_state, State::Done(_)));
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], Effect::ShowMessage(_)));
    }

    #[test]
    fn test_error_action() {
        let error_msg = "Test error".to_string();
        let (new_state, effects) = update(State::Initializing, Action::Error(error_msg.clone()));

        assert_eq!(new_state, State::Error(error_msg.clone()));
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], Effect::ShowError(_)));
    }

    #[test]
    fn test_invalid_transitions() {
        // Test invalid action for state
        let (new_state, effects) = update(
            State::Done(WriteResult {
                entry_id: Uuid::new_v4(),
                entry_path: "test".to_string(),
                session_completed: true,
            }),
            Action::UserResponse("test".to_string()),
        );

        assert!(matches!(new_state, State::Error(_)));
        assert_eq!(effects.len(), 1);
        assert!(matches!(effects[0], Effect::ShowError(_)));
    }
}
