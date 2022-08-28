mod message;
mod shard;

use crate::types::Context;
use crate::types::ThangResult;
use futures::StreamExt;
use log::error;
use message::on_message;
use shard::on_shard_connect;
use std::{future::Future, sync::Arc};
use twilight_gateway::{cluster::Events, Event};

use std::{error::Error, fmt};

#[derive(Debug)]
struct Thing;

impl Error for Thing {}

impl fmt::Display for Thing {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Oh no, something bad went down")
    }
}

pub async fn iterate_websocket(mut events: Events, context: Arc<Context>) -> ThangResult<()> {
    while let Some((shard_id, event)) = events.next().await {
        tokio::spawn(handle_event(shard_id, event, context.clone()));
    }

    Ok(())
}

async fn handle_event(shard_id: u64, event: Event, context: Arc<Context>) {
    match event {
        Event::ShardConnected(_) => on_shard_connect(shard_id),
        Event::MessageCreate(msg) => async_wrapper(on_message(*msg, context), "message_create"),
        _ => {}
    };
}

fn async_wrapper(todo: impl Future<Output = ThangResult<()>> + Send + 'static, name: &'static str) {
    tokio::spawn(async move {
        if let Err(e) = todo.await {
            error!("Failed to execute {} handler: {}", name, e);
        }
    });
}
