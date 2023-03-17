use serde::{Deserialize, Serialize};

use super::{Member, Message, User};

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
// rustfmt bad, it do funny things to my structs :(
#[rustfmt::skip]
pub enum Event {
    Authenticate { token: String },
    Ping { data: Vec<u8> },
    Error { error: ErrorType },
    Authenticated,
    Bulk { v: Vec<Event> },
    Pong { data: Vec<u8> },
    Message(Message),
    Ready { users: Vec<User>, members: Vec<Member> },
}
