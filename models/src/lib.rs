use serde::{Deserialize, Serialize};
use std::error::Error;
use std::result::Result as StdResult;

pub type Result<T> = StdResult<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Serialize, Deserialize)]
pub struct Event<'a> {
    pub platform: &'a str,
    pub data: EventData,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t")]
pub enum EventData {
    MessageCreate(Message),
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Message {
    pub content: String,
    pub author: String,
    pub attachments: Vec<String>,
    pub replies: Vec<Reply>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Reply {
    pub content: String,
    pub author: String,
}
