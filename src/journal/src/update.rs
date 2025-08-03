use crate::action::Action;
use crate::effects::Effect;
use crate::state::{JournalSession, SessionMode, Speaker, State, WriteResult};
use uuid::Uuid;

pub fn update(state: State, action: Action) -> (State, Vec<Effect>) {
    match (state, action) {
        // Starting a new journal session
        (State::Initializing, Action::Start) => {
            (State::PromptingForNew, vec![])
        }

        // Resuming an existing session
        (State::Initializing, Action::Resume(session_id)) => {
            (State::Initializing, vec![Effect::LoadSession(session_id)])
        }

        // Mode selection
        (State::PromptingForNew, Action::SelectMode(mode)) => {
            let mut session = JournalSession::new(mode);
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

            // Note: The session document UUID will be created during SaveSession effect
            // The UpdateIndex effect will need to be triggered after the document is saved
            (
                State::InSession(session.clone()),
                vec![Effect::SaveSession(session.clone())],
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
                vec![Effect::SaveSession(session.clone())],
            )
        }

        // Moving to next question
        (State::InSession(session), Action::NextQuestion) => {
            let user_responses = session.get_user_responses().len();
            let questions = session.mode.get_initial_questions();

            if user_responses < questions.len() {
                (State::InSession(session), vec![])
            } else {
                (State::InSession(session), vec![Effect::PromptForUserInput])
            }
        }

        // Stopping session (user pressed 's')
        (State::InSession(mut session), Action::Stop) => {
            session.mark_completed();

            (
                State::Analyzing(session.clone()),
                vec![
                    Effect::SaveSession(session.clone()),
                    Effect::GenerateAnalysis {
                        session: session.clone(),
                    },
                ],
            )
        }

        // Analysis completed - show analysis and create final entry
        (State::Analyzing(session), Action::AnalysisComplete(analysis)) => {
            let entry_id = Uuid::new_v4();
            (
                State::AnalysisReady { 
                    session: session.clone(), 
                    analysis: analysis.clone() 
                },
                vec![
                    Effect::CreateFinalEntry {
                        session,
                        entry_id,
                        analysis,
                    },
                ],
            )
        }

        // Final entry created successfully (legacy handler - should not be used anymore)
        (State::Analyzing(_), Action::Stop) => {
            let entry_id = Uuid::new_v4();
            (
                State::Done(WriteResult {
                    entry_id,
                    entry_path: format!("entry_{entry_id}.md"),
                    session_completed: true,
                }),
                vec![Effect::ClearIndex],
            )
        }

        // Final entry created successfully
        (State::AnalysisReady { .. }, Action::FinalEntryCreated { entry_path, .. }) => {
            let entry_id = Uuid::new_v4();
            (
                State::Done(WriteResult {
                    entry_id,
                    entry_path,
                    session_completed: true,
                }),
                vec![Effect::ClearIndex],
            )
        }
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
            (State::Error(error_msg), vec![])
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
        assert_eq!(effects.len(), 0);
    }

    #[test]
    fn test_mode_selection() {
        let (new_state, effects) = update(
            State::PromptingForNew,
            Action::SelectMode(SessionMode::Morning),
        );

        assert!(matches!(new_state, State::InSession(_)));
        assert_eq!(effects.len(), 1);
        // Only SaveSession effect
        assert!(matches!(effects[0], Effect::SaveSession(_)));
    }

    #[test]
    fn test_user_response() {
        let mut session = JournalSession::new(SessionMode::Morning);
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
    fn test_stop_session() {
        let session = JournalSession::new(SessionMode::Morning);
        let initial_state = State::InSession(session.clone());

        let (new_state, effects) = update(initial_state, Action::Stop);

        assert!(matches!(new_state, State::Analyzing(_)));
        assert_eq!(effects.len(), 2);
        assert!(matches!(effects[0], Effect::SaveSession(_)));
        assert!(matches!(effects[1], Effect::GenerateAnalysis { .. }));
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
        assert_eq!(effects.len(), 0);
    }
}
