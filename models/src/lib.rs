use serde::{Deserialize, Serialize};
use todel::models::Message as EludrisMessage;
use twilight_model::channel::Message as DiscordMessage;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "platform")]
pub enum Message {
    Eludris(EludrisMessage),
    Discord(DiscordMessage),
}
