use optional_struct::Applyable;
use serde::{Deserialize, Serialize};

// Not all fields are here.
// https://github.com/revoltchat/backend/blob/master/crates/quark/src/models/channels/message.rs
#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "snake_case")]
#[optional_struct]
#[skip_serializing_none]
pub struct Message {
    #[serde(rename = "_id")]
    pub id: String,
    pub channel: String,
    pub author: String,
    pub content: Option<String>,
}
