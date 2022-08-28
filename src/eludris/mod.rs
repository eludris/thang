use crate::types::{Context, Message, WsReader};
use futures::StreamExt;
use std::sync::Arc;

pub async fn iterate_websocket(mut reader: WsReader, context: Arc<Context>) {
    while let Some(message) = reader.next().await {
        let data = message.unwrap().into_text().unwrap();
        let msg: Message = serde_json::from_str(&data).unwrap();

        context
            .http
            .execute_webhook(context.bridge_webhook_id, &context.bridge_webhook_token)
            .content(&msg.content)
            .unwrap()
            .username(&msg.author)
            .unwrap()
            .exec()
            .await
            .unwrap();
    }
}
