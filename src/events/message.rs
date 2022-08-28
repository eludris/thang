use crate::types::{Context, ThangResult};
use reqwest::StatusCode;
use serde_json::json;
use std::sync::Arc;
use twilight_model::gateway::payload::incoming::MessageCreate;

pub async fn on_message(msg: MessageCreate, context: Arc<Context>) -> ThangResult<()> {
    let username = &msg.author.name;
    let author = match msg.member.as_ref() {
        Some(member) => member.nick.as_ref().unwrap_or(username),
        None => username,
    };
    let content = &msg.content;

    let response = context
        .eludris_http_client
        .post(&context.eludris_rest_url)
        .json(&json!({"author": author, "content": content}))
        .send()
        .await?;

    if let StatusCode::OK = response.status() {
        panic!(
            "{:?} failed with status code {}",
            response,
            response.status()
        );
    }

    Ok(())
}
