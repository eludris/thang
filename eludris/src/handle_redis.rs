// May seem unreachable now, but not when more items are added.
#![allow(unreachable_patterns)]
use eludrs::HttpClient;
use futures::StreamExt;
use models::DiscordEvent;
use models::Event;
use models::ThangResult;
use redis::aio::Connection;
use serde::{Deserialize, Serialize};
use twilight_model::id::{marker::ChannelMarker, Id};

#[derive(Debug, Serialize, Deserialize)]
struct RatelimitResponse {
    data: RatelimitData,
}

#[derive(Debug, Serialize, Deserialize)]
struct RatelimitData {
    retry_after: u64,
}

pub async fn handle_redis(
    redis: Connection,
    rest: HttpClient,
    discord_bridge_channel_id: Id<ChannelMarker>,
) -> ThangResult<()> {
    let mut pubsub = redis.into_pubsub();

    pubsub.subscribe("thang-bridge").await?;
    let mut pubsub = pubsub.into_on_message();

    while let Some(payload) = pubsub.next().await {
        // TODO: handle more of the errors here
        let payload: String = match payload.get_payload() {
            Ok(payload) => payload,
            Err(err) => {
                log::error!("Could not get pubsub payload: {}", err);
                continue;
            }
        };
        let payload: Event = match serde_json::from_str(&payload) {
            Ok(payload) => payload,
            Err(err) => {
                log::error!("Failed to deserialize event payload: {}", err);
                continue;
            }
        };
        match payload {
            Event::Discord(DiscordEvent::MessageCreate(msg)) => {
                if msg.channel_id != discord_bridge_channel_id {
                    continue;
                }

                let username = &msg.author.name;
                let name = match msg.member.as_ref() {
                    Some(member) => member.nick.as_ref().unwrap_or(username),
                    None => username,
                };
                let mut name = format!("Bridge-{}", name);
                if name.len() > 32 {
                    name = name.drain(..32).collect();
                }

                let mut content = msg.content.clone();

                if let Some(referenced) = &msg.referenced_message {
                    let mut reply = referenced
                        .content
                        .lines()
                        .map(|l| format!("> {}", l))
                        .collect::<Vec<String>>()
                        .join("\n");
                    let username = &referenced.author.name;
                    let mut name = match referenced.member.as_ref() {
                        Some(member) => member.nick.as_ref().unwrap_or(username),
                        None => username,
                    }
                    .clone();
                    if name.len() > 32 {
                        name = name.drain(..32).collect();
                    }
                    reply.push_str(&format!("\n@{}", name));
                    content = format!("\n{}\n{}", reply, content);
                }

                let attachments = msg
                    .attachments
                    .iter()
                    .map(|a| a.url.as_ref())
                    .collect::<Vec<&str>>()
                    .join("\n");

                if !attachments.is_empty() {
                    if !content.is_empty() {
                        content.push('\n');
                    }
                    content.push_str(&attachments);
                }
                rest.send_message(name, &content).await?;
            }
            // Eludris does not have anything other than message create.
            Event::Discord(_) => {}
            Event::Eludris(_) => {}
            payload => {
                log::info!("Unhandled payload from pubsub: {:?}", payload)
            }
        }
    }

    Ok(())
}