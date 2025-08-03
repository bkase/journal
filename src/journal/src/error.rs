use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Structured error types for the journal application
#[derive(Error, Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Error {
    #[error("Aethel operation failed: {message}")]
    Aethel { message: String },
    
    #[error("IO operation failed: {message}")]
    Io { message: String },
    
    #[error("JSON serialization failed: {message}")]
    Json { message: String },
    
    #[error("AI analysis failed: {0}")]
    AiAnalysis(String),
    
    #[error("Session not found: {session_id}")]
    SessionNotFound { session_id: String },
    
    #[error("Invalid session state: {reason}")]
    InvalidSessionState { reason: String },
    
    #[error("Vault operation failed: {operation}")]
    VaultOperation { operation: String },
    
    #[error("Claude CLI execution failed: {message}")]
    ClaudeExecution { message: String },
    
    #[error("Configuration error: {0}")]
    Config(String),
    
    #[error("User input error: {0}")]
    UserInput(String),
    
    #[error("System error: {0}")]
    System(String),
}

impl Error {
    /// Create an AI analysis error
    pub fn ai_analysis<S: Into<String>>(message: S) -> Self {
        Error::AiAnalysis(message.into())
    }
    
    /// Create a session not found error
    pub fn session_not_found<S: Into<String>>(session_id: S) -> Self {
        Error::SessionNotFound {
            session_id: session_id.into(),
        }
    }
    
    /// Create an invalid session state error
    pub fn invalid_session_state<S: Into<String>>(reason: S) -> Self {
        Error::InvalidSessionState {
            reason: reason.into(),
        }
    }
    
    /// Create a vault operation error
    pub fn vault_operation<S: Into<String>>(operation: S) -> Self {
        Error::VaultOperation {
            operation: operation.into(),
        }
    }
    
    /// Create a Claude execution error
    pub fn claude_execution<S: Into<String>>(message: S) -> Self {
        Error::ClaudeExecution {
            message: message.into(),
        }
    }
    
    /// Create a configuration error
    pub fn config<S: Into<String>>(message: S) -> Self {
        Error::Config(message.into())
    }
    
    /// Create a user input error
    pub fn user_input<S: Into<String>>(message: S) -> Self {
        Error::UserInput(message.into())
    }
    
    /// Create a system error
    pub fn system<S: Into<String>>(message: S) -> Self {
        Error::System(message.into())
    }
    
    /// Check if this error is recoverable (non-fatal)
    pub fn is_recoverable(&self) -> bool {
        matches!(
            self,
            Error::AiAnalysis(_) | Error::UserInput(_) | Error::ClaudeExecution { .. }
        )
    }
    
    /// Check if this error requires fallback behavior
    pub fn needs_fallback(&self) -> bool {
        matches!(self, Error::AiAnalysis(_) | Error::ClaudeExecution { .. })
    }
}

/// Convert from AethelCoreError to our structured Error type
impl From<aethel_core::AethelCoreError> for Error {
    fn from(err: aethel_core::AethelCoreError) -> Self {
        Error::Aethel {
            message: err.to_string(),
        }
    }
}

/// Convert from std::io::Error to our structured Error type
impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Error::Io {
            message: err.to_string(),
        }
    }
}

/// Convert from serde_json::Error to our structured Error type
impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Error::Json {
            message: err.to_string(),
        }
    }
}

/// Convert from anyhow::Error to our structured Error type
impl From<anyhow::Error> for Error {
    fn from(err: anyhow::Error) -> Self {
        // Check for error types by examining the error chain
        let error_chain = format!("{:#}", err);
        
        if error_chain.contains("aethel") || error_chain.contains("vault") {
            return Error::VaultOperation {
                operation: error_chain,
            };
        }
        
        if error_chain.contains("claude") || error_chain.contains("AI") {
            return Error::AiAnalysis(error_chain);
        }
        
        if error_chain.contains("JSON") || error_chain.contains("serde") {
            return Error::System(format!("JSON error: {}", error_chain));
        }
        
        // Default to system error
        Error::System(error_chain)
    }
}