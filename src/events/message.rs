use crate::types::{ContextT, ThangResult};
use reqwest::StatusCode;
use serde_json::json;
use twilight_model::gateway::payload::incoming::MessageCreate;

pub async fn on_message(msg: MessageCreate, context: ContextT) -> ThangResult<()> {
    let username = &msg.author.name;
    let author = match msg.member.as_ref() {
        Some(member) => member.nick.as_ref().unwrap_or(username),
        None => username,
    };
    let content = &msg.content;

    if !author.starts_with("Bridge-")
        && msg.channel_id == context.bridge_channel_id
        && msg.author.id != context.bridge_webhook_id.cast()
    {
        let response = context
            .eludris_http_client
            .post(&context.eludris_rest_url)
            .json(&json!({"author": format!("Bridge-{}", author), "content": content}))
            .send()
            .await?;

        if response.status() != StatusCode::OK {
            panic!(
                "{:?} failed with status code {}",
                response,
                response.status()
            );
        }
    }

    Ok(())
}
