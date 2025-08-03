use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum State {
    Initializing,
    PromptingForNew,
    InSession(JournalSession),
    Analyzing(JournalSession),
    AnalysisReady { session: JournalSession, analysis: String },
    Done(WriteResult),
    Error(String),
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct JournalSession {
    pub mode: SessionMode,
    pub transcript: Vec<TranscriptEntry>,
    pub metadata: SessionMetadata,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SessionMode {
    Morning,
    Evening,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TranscriptEntry {
    pub timestamp: DateTime<Utc>,
    pub speaker: Speaker,
    pub content: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum Speaker {
    User,
    Coach,
    System,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionMetadata {
    pub session_doc_id: Option<Uuid>,
    pub final_entry_id: Option<Uuid>,
    pub completed_at: Option<DateTime<Utc>>,
    pub custom_fields: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WriteResult {
    pub entry_id: Uuid,
    pub entry_path: String,
    pub session_completed: bool,
}

impl State {
    pub fn is_terminal(&self) -> bool {
        matches!(self, State::Done(_) | State::Error(_))
    }

    pub fn is_interactive(&self) -> bool {
        matches!(self, State::PromptingForNew | State::InSession(_))
    }
}

impl SessionMode {
    pub fn get_initial_questions(&self) -> Vec<&'static str> {
        match self {
            SessionMode::Morning => vec![
                "How are you feeling as you start this day?",
                "What's your energy level right now?",
                "What are you most looking forward to today?",
                "Is there anything weighing on your mind this morning?",
            ],
            SessionMode::Evening => vec![
                "How was your day overall?",
                "What went well today?",
                "What was challenging?",
                "How are you feeling as you wind down?",
                "What are you grateful for today?",
            ],
        }
    }

    pub fn get_coaching_context(&self) -> &'static str {
        match self {
            SessionMode::Morning => {
                "You are an empathetic journaling coach helping someone start their day with intention and awareness. Ask follow-up questions that help them explore their feelings, set intentions, and prepare mentally for the day ahead. Be warm, supportive, and gently curious."
            }
            SessionMode::Evening => {
                "You are an empathetic journaling coach helping someone reflect on their day and process their experiences. Ask follow-up questions that help them explore what they learned, how they grew, and what they want to carry forward. Be warm, supportive, and help them find meaning in their experiences."
            }
        }
    }
}

impl JournalSession {
    pub fn new(mode: SessionMode) -> Self {
        Self {
            mode,
            transcript: Vec::new(),
            metadata: SessionMetadata {
                session_doc_id: None,
                final_entry_id: None,
                completed_at: None,
                custom_fields: HashMap::new(),
            },
        }
    }

    pub fn add_entry(&mut self, speaker: Speaker, content: String) {
        self.transcript.push(TranscriptEntry {
            timestamp: Utc::now(),
            speaker,
            content,
        });
    }

    pub fn get_user_responses(&self) -> Vec<&TranscriptEntry> {
        self.transcript
            .iter()
            .filter(|entry| matches!(entry.speaker, Speaker::User))
            .collect()
    }

    pub fn get_conversation_summary(&self) -> String {
        let mut summary = format!(
            "Journal Session ({})\n\n",
            match self.mode {
                SessionMode::Morning => "Morning",
                SessionMode::Evening => "Evening",
            }
        );

        for entry in &self.transcript {
            let speaker_label = match entry.speaker {
                Speaker::User => "You",
                Speaker::Coach => "Coach",
                Speaker::System => "System",
            };
            summary.push_str(&format!("{}: {}\n\n", speaker_label, entry.content));
        }

        summary
    }

    pub fn mark_completed(&mut self) {
        self.metadata.completed_at = Some(Utc::now());
    }
}
