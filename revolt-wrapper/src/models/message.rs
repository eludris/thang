// use optional_struct::Applyable;
use crate::models::File;
use serde::{Deserialize, Serialize};

// Not all fields are here.
// https://github.com/revoltchat/backend/blob/master/crates/quark/src/models/channels/message.rs
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    pub channel: String,
    pub author: String,
    pub content: Option<String>,
    pub attachments: Option<Vec<File>>,
    pub replies: Option<Vec<String>>,
    pub masquerade: Option<Masquerade>,
}

/// Name and / or avatar override information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Masquerade {
    pub name: Option<String>,
    pub avatar: Option<String>,
}
