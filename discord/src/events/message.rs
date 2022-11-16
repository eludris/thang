use crate::types::{ContextT, ThangResult};
use crate::util::get_avatar_url;
use redis::Commands;
use serde::{Deserialize, Serialize};
use serde_json::to_string;
use models::{Attachment, Message};
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
    let context_r = context.read().await;

    let username = &msg.author.name;
    let author = match msg.member.as_ref() {
        Some(member) => member.nick.as_ref().unwrap_or(username),
        None => username,
    };
    let avatar = get_avatar_url(&msg);
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
        && msg.channel_id == context_r.bridge_channel_id
        && msg.author.id != context_r.bridge_webhook_id.cast()
        // Possible thanks to attachments and embeds
        && !content.is_empty()
        && username.len() + 7 < 32
    {
        let mut context = context.write().await;

        context.redis.publish::<&str, String, String>(
            "messages",
            to_string(&Message {
                content,
                author: author.to_string(),
                avatar,
                attachments: msg
                    .attachments
                    .iter()
                    .map(|a| Attachment {
                        url: a.url.clone(),
                        name: a.filename.clone(),
                    })
                    .collect(),
            })?,
        )?;
    }

    Ok(())
}
