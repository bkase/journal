use crate::state::SessionMode;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
pub enum Action {
    Start,
    Resume(Uuid),
    SelectMode(SessionMode),
    UserResponse(String),
    CoachResponse(String),
    NextQuestion,
    Stop,
    AnalysisComplete(String),
    FinalEntryCreated {
        entry_path: String,
        analysis: String,
    },
}

impl Action {}

#[derive(Debug, Clone, PartialEq)]
pub struct UserInput {
    pub raw_input: String,
    pub processed: Action,
}

#[derive(Debug, Clone, PartialEq)]
pub enum InputContext {
    ModeSelection,
    InSession,
}

impl UserInput {
    pub fn new_with_context(input: String, context: InputContext) -> Self {
        let processed = Self::parse_input(&input, context);
        Self {
            raw_input: input,
            processed,
        }
    }

    fn parse_input(input: &str, context: InputContext) -> Action {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Action::NextQuestion;
        }

        match context {
            InputContext::ModeSelection => match trimmed.to_lowercase().as_str() {
                "morning" | "m" => Action::SelectMode(SessionMode::Morning),
                "evening" | "e" => Action::SelectMode(SessionMode::Evening),
                _ => Action::UserResponse(trimmed.to_string()),
            },
            InputContext::InSession => match trimmed.to_lowercase().as_str() {
                "s" | "stop" => Action::Stop,
                _ => Action::UserResponse(trimmed.to_string()),
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_input_parsing_mode_selection() {
        let cases = vec![
            ("", Action::NextQuestion),
            ("   ", Action::NextQuestion),
            ("morning", Action::SelectMode(SessionMode::Morning)),
            ("m", Action::SelectMode(SessionMode::Morning)),
            ("evening", Action::SelectMode(SessionMode::Evening)),
            ("e", Action::SelectMode(SessionMode::Evening)),
            (
                "I feel great today!",
                Action::UserResponse("I feel great today!".to_string()),
            ),
            (
                "random text",
                Action::UserResponse("random text".to_string()),
            ),
        ];

        for (input, expected) in cases {
            let user_input =
                UserInput::new_with_context(input.to_string(), InputContext::ModeSelection);
            assert_eq!(
                user_input.processed, expected,
                "Failed for input: '{input}' in mode selection context"
            );
        }
    }

    #[test]
    fn test_user_input_parsing_in_session() {
        let cases = vec![
            ("", Action::NextQuestion),
            ("   ", Action::NextQuestion),
            ("s", Action::Stop),
            ("stop", Action::Stop),
            ("S", Action::Stop), // Test case insensitive
            ("STOP", Action::Stop),
            ("e", Action::UserResponse("e".to_string())),
            ("evening", Action::UserResponse("evening".to_string())),
            ("morning", Action::UserResponse("morning".to_string())),
            ("m", Action::UserResponse("m".to_string())),
            (
                "I feel great today!",
                Action::UserResponse("I feel great today!".to_string()),
            ),
        ];

        for (input, expected) in cases {
            let user_input =
                UserInput::new_with_context(input.to_string(), InputContext::InSession);
            assert_eq!(
                user_input.processed, expected,
                "Failed for input: '{input}' in session context"
            );
        }
    }
}
