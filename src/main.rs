mod commands;

use futures_util::StreamExt;
use std::{env, error::Error, sync::Arc};
use twilight_cache_inmemory::{InMemoryCache, ResourceType};
use twilight_gateway::{Cluster, Event};
use twilight_http::Client;
use twilight_model::gateway::{payload::incoming::MessageCreate, Intents};

const PREFIX: &str = "-";

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    dotenv::dotenv()?;
    let token = env::var("TOKEN")?;

    let (cluster, mut events) = Cluster::new(
        token.to_owned(),
        Intents::GUILD_MESSAGES | Intents::MESSAGE_CONTENT,
    )
    .await?;
    let cluster = Arc::new(cluster);

    let cluster_spawm = Arc::clone(&cluster);

    tokio::spawn(async move {
        cluster_spawm.up().await;
    });

    let http = Arc::new(Client::new(token));

    let cache = InMemoryCache::builder()
        .resource_types(ResourceType::MESSAGE)
        .build();

    while let Some((shard_id, event)) = events.next().await {
        cache.update(&event);

        tokio::spawn(handle_event(shard_id, event, Arc::clone(&http)));
    }

    Ok(())
}

async fn handle_event(
    shard_id: u64,
    event: Event,
    http: Arc<Client>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    match event {
        Event::ShardConnected(_) => println!("Shard {} connected", shard_id),
        Event::Ready(ready) => println!(
            "Connected as {}#{}",
            ready.user.name, ready.user.discriminator
        ),
        Event::MessageCreate(msg) => match handle_message(&http, &msg).await {
            Ok(_) => {}
            Err(error_message) => {
                http.create_message(msg.channel_id)
                    .content(&format!(
                        "Oh no, an error occured:\n```rust\n{:#?}\n```",
                        error_message
                    ))?
                    .exec()
                    .await?;
            }
        },
        Event::InteractionCreate(interaction) => println!("{:#?}", interaction),
        _ => {}
    }

    Ok(())
}

async fn handle_message(
    http: &Arc<Client>,
    msg: &Box<MessageCreate>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    if !msg.content.starts_with(PREFIX) {
        return Ok(());
    }
    let content = &msg.content[PREFIX.len()..];
    let cmd: &str;
    let args: Vec<&str>;
    if let Some((start, rest)) = content.split_once(" ") {
        cmd = start;
        args = rest.split(" ").collect();
    } else {
        cmd = content;
        args = vec![];
    };
    match cmd {
        "ping" => commands::misc::ping(http, msg, args).await?,
        "say" => commands::misc::say(http, msg, args).await?,
        "poll" => commands::misc::poll(http, msg, args).await?,
        _ => {}
    };
    Ok(())
}
