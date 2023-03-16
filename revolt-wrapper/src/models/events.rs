use serde::{Deserialize, Serialize};

use super::Message;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorType {
    LabelMe,
    InternalError,
    InvalidSession,
    OnboardingNotFinished,
    AlreadyAuthenticated,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum Event {
    Authenticate { token: String },
    Ping { data: Vec<u8> },
    Error { error: ErrorType },
    Authenticated,
    Bulk { v: Vec<Event> },
    Pong { data: Vec<u8> },
    Message(Message),
}
