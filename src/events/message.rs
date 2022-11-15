use std::time::Duration;

use crate::types::{ContextT, ThangResult};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::json;
use tokio::time;
use twilight_model::gateway::payload::incoming::MessageCreate;

#[derive(Debug, Serialize, Deserialize)]
struct RatelimitResponse {
    data: RatelimitData,
}

#[derive(Debug, Serialize, Deserialize)]
struct RatelimitData {
    retry_after: u64,
}

pub async fn on_message(msg: MessageCreate, context: ContextT) -> ThangResult<()> {
    let username = &msg.author.name;
    let author = match msg.member.as_ref() {
        Some(member) => member.nick.as_ref().unwrap_or(username),
        None => username,
    };
    let mut content = msg.content.clone();

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

    if !author.starts_with("Bridge-")
        && msg.channel_id == context.bridge_channel_id
        && msg.author.id != context.bridge_webhook_id.cast()
        // Possible thanks to attachments and embeds
        && !content.is_empty()
        && username.len() + 7 < 32
    {
        loop {
            let response = context
                .eludris_http_client
                .post(format!("{}/messages/", context.eludris_rest_url))
                .json(&json!({"author": format!("Bridge-{}", author), "content": content}))
                .send()
                .await?;

            match response.status() {
                StatusCode::OK => break,
                StatusCode::TOO_MANY_REQUESTS => {
                    let RatelimitResponse {
                        data: RatelimitData { retry_after },
                    } = response.json().await?;
                    log::warn!(
                        "Bridge eludris user is ratelimited, waiting {}s",
                        retry_after
                    );
                    time::sleep(Duration::from_secs(retry_after)).await;
                }
                _ => {
                    log::warn!(
                        "{:?} failed with status code {}",
                        response,
                        response.status()
                    );
                    break;
                }
            }
        }
    }

    Ok(())
}
