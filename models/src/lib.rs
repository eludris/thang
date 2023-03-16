use revolt_wrapper::models::Event as RevoltEvent;
use serde::{Deserialize, Serialize};
use std::error::Error;
use todel::models::Payload as EludrisEvent;
use twilight_model::gateway::payload::incoming::*;

pub type ThangResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "platform")]
pub enum Event {
    Eludris(EludrisEvent),
    Discord(DiscordEvent),
    Revolt(RevoltEvent),
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "t", content = "d")]
pub enum DiscordEvent {
    ChannelPinsUpdate(ChannelPinsUpdate),
    ChannelUpdate(Box<ChannelUpdate>),
    MessageCreate(Box<MessageCreate>),
    MessageDelete(MessageDelete),
    MessageDeleteBulk(MessageDeleteBulk),
    MessageUpdate(Box<MessageUpdate>),
}
