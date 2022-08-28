use crate::types::{Context, ThangResult};
use reqwest::StatusCode;
use std::{collections::HashMap, sync::Arc};
use twilight_model::gateway::payload::incoming::MessageCreate;

pub async fn on_message(msg: MessageCreate, context: Arc<Context>) -> ThangResult<()> {
    let author = &msg
        .member
        .as_ref()
        .unwrap()
        .nick
        .as_ref()
        .unwrap_or(&msg.author.name);
    let content = &msg.content;

    let mut map: HashMap<&str, &str> = HashMap::new();
    map.insert("author", author);
    map.insert("content", content);
    let response = context
        .eludris_http_client
        .post(&context.eludris_rest_url)
        .json(&map)
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
