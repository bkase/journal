use crate::state::SessionMode;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // Some variants are for future use
pub enum Action {
    Start,
    Resume(Uuid),
    SelectMode(SessionMode),
    UserResponse(String),
    CoachResponse(String),
    NextQuestion,
    RequestSummary,
    Save,
    Complete,
    Quit,
    Error(String),
}

impl Action {
    #[allow(dead_code)] // For future use
    pub fn requires_user_input(&self) -> bool {
        matches!(self, Action::SelectMode(_) | Action::UserResponse(_))
    }

    #[allow(dead_code)] // For future use
    pub fn is_session_action(&self) -> bool {
        matches!(
            self,
            Action::UserResponse(_)
                | Action::CoachResponse(_)
                | Action::NextQuestion
                | Action::RequestSummary
                | Action::Save
                | Action::Complete
        )
    }

    #[allow(dead_code)] // For future use
    pub fn is_terminal(&self) -> bool {
        matches!(self, Action::Quit | Action::Error(_))
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct UserInput {
    pub raw_input: String,
    pub processed: Action,
}

impl UserInput {
    pub fn new(input: String) -> Self {
        let processed = Self::parse_input(&input);
        Self {
            raw_input: input,
            processed,
        }
    }

    fn parse_input(input: &str) -> Action {
        let trimmed = input.trim();

        if trimmed.is_empty() {
            return Action::NextQuestion;
        }

        match trimmed.to_lowercase().as_str() {
            "quit" | "exit" | "q" => Action::Quit,
            "save" | "s" => Action::Save,
            "done" | "complete" | "finish" => Action::Complete,
            "summary" | "sum" => Action::RequestSummary,
            "morning" | "m" => Action::SelectMode(SessionMode::Morning),
            "evening" | "e" => Action::SelectMode(SessionMode::Evening),
            _ => Action::UserResponse(trimmed.to_string()),
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
#[allow(dead_code)] // For future use
pub enum SessionCommand {
    Continue,
    Pause,
    Save,
    Complete,
    Quit,
}

impl SessionCommand {
    #[allow(dead_code)] // For future use
    pub fn from_input(input: &str) -> Option<Self> {
        match input.trim().to_lowercase().as_str() {
            "continue" | "c" => Some(Self::Continue),
            "pause" | "p" => Some(Self::Pause),
            "save" | "s" => Some(Self::Save),
            "done" | "complete" | "finish" => Some(Self::Complete),
            "quit" | "exit" | "q" => Some(Self::Quit),
            _ => None,
        }
    }

    #[allow(dead_code)] // For future use
    #[allow(clippy::wrong_self_convention)] // This method is intended to consume self
    pub fn to_action(self) -> Action {
        match self {
            Self::Continue => Action::NextQuestion,
            Self::Pause => Action::Save,
            Self::Save => Action::Save,
            Self::Complete => Action::Complete,
            Self::Quit => Action::Quit,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_user_input_parsing() {
        let cases = vec![
            ("", Action::NextQuestion),
            ("   ", Action::NextQuestion),
            ("quit", Action::Quit),
            ("QUIT", Action::Quit),
            ("q", Action::Quit),
            ("save", Action::Save),
            ("s", Action::Save),
            ("done", Action::Complete),
            ("morning", Action::SelectMode(SessionMode::Morning)),
            ("m", Action::SelectMode(SessionMode::Morning)),
            ("evening", Action::SelectMode(SessionMode::Evening)),
            ("e", Action::SelectMode(SessionMode::Evening)),
            (
                "I feel great today!",
                Action::UserResponse("I feel great today!".to_string()),
            ),
            ("summary", Action::RequestSummary),
        ];

        for (input, expected) in cases {
            let user_input = UserInput::new(input.to_string());
            assert_eq!(
                user_input.processed, expected,
                "Failed for input: '{input}'"
            );
        }
    }

    #[test]
    fn test_action_properties() {
        assert!(Action::Quit.is_terminal());
        assert!(Action::Error("test".to_string()).is_terminal());
        assert!(!Action::Start.is_terminal());

        assert!(Action::UserResponse("test".to_string()).is_session_action());
        assert!(Action::Save.is_session_action());
        assert!(!Action::Start.is_session_action());

        assert!(Action::UserResponse("test".to_string()).requires_user_input());
        assert!(!Action::Save.requires_user_input());
    }

    #[test]
    fn test_session_command_parsing() {
        let cases = vec![
            ("continue", Some(SessionCommand::Continue)),
            ("c", Some(SessionCommand::Continue)),
            ("pause", Some(SessionCommand::Pause)),
            ("save", Some(SessionCommand::Save)),
            ("quit", Some(SessionCommand::Quit)),
            ("invalid", None),
        ];

        for (input, expected) in cases {
            let result = SessionCommand::from_input(input);
            assert_eq!(result, expected, "Failed for input: '{input}'");
        }
    }
}
