use crate::types::{ContextT, ThangResult};
use thang_types::Message;
use twilight_model::channel::message::Embed;
use twilight_util::builder::embed::{EmbedBuilder, ImageSource};

pub async fn receive_redis(redis: redis::Client, context: ContextT) -> ThangResult<()> {
    let mut redis_connection = redis.get_connection()?;
    let mut pubsub = redis_connection.as_pubsub();

    pubsub.subscribe("messages")?;

    loop {
        let msg = pubsub
            .get_message()?
            .get_payload()
            .map(|s: String| serde_json::from_str::<Message>(&s))??;

        let context = context.lock().await;
        context
            .http
            .execute_webhook(
                context.bridge_webhook_id,
                context.bridge_webhook_token.as_str(),
            )
            .content(&msg.content.to_string())?
            .username(&format!("Bridge-{}", msg.author).to_string())?
            .embeds(
                &msg.attachments
                    .iter()
                    .map(|a| {
                        EmbedBuilder::new()
                            .title(a.name.clone())
                            .image(ImageSource::url(a.url.clone()).unwrap_or_else(|_| {
                                ImageSource::url("https://example.com").unwrap()
                            }))
                            .build()
                    })
                    .collect::<Vec<Embed>>(),
            )?
            .await?;
    }
}
