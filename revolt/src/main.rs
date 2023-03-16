use std::{
    env,
    time::{SystemTime, UNIX_EPOCH},
};

use futures::StreamExt;
use models::ThangResult;
use revolt_wrapper::{Event, GatewayClient};

#[tokio::main]
async fn main() -> ThangResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let token = env::var("REVOLT_TOKEN").expect("REVOLT_TOKEN must be set");

    let client = GatewayClient::new(token);

    let mut events = client.get_events().await?;

    log::info!("Connected to gateway!");

    while let Some(msg) = events.next().await {
        match msg {
            Event::Message(msg) => {
                log::info!("Received message: {:?}", msg);
            }
            Event::Pong { data } => {
                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .expect("Time went backwards")
                    .as_millis();
                log::info!(
                    "Received pong! Latency is {}ms",
                    now - u128::from_be_bytes(data.try_into().unwrap())
                );
            }
            _ => {}
        }
    }

    Ok(())
}
