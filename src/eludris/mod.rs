use crate::types::{Context, Message, ThangResult, WsReader};
use futures::StreamExt;
use std::sync::Arc;
use twilight_validate::message::MESSAGE_CONTENT_LENGTH_MAX;
use twilight_validate::request::webhook_username as validate_webhook_username;

pub async fn iterate_websocket(mut reader: WsReader, context: Arc<Context>) -> ThangResult<()> {
    while let Some(message) = reader.next().await {
        let data = message.unwrap().into_text().unwrap();
        let msg: Message = serde_json::from_str(&data).unwrap();

        if !msg.author.starts_with("Bridge-") {
            let username = match validate_webhook_username(&msg.author) {
                Ok(_) => &msg.author,
                Err(_) => "Eludris Bridge",
            };

            let truncated = format!("{}...", &msg.content[..MESSAGE_CONTENT_LENGTH_MAX - 3]);
            let content = match msg.content.chars().count() <= MESSAGE_CONTENT_LENGTH_MAX {
                true => msg.content,
                false => truncated,
            };

            context
                .http
                .execute_webhook(context.bridge_webhook_id, &context.bridge_webhook_token)
                .content(&content)
                .unwrap()
                .username(username)
                .unwrap()
                .exec()
                .await?;
        }
    }

    Ok(())
}
