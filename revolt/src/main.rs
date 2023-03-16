use std::env;

use futures::StreamExt;
use models::ThangResult;
use revolt_wrapper::GatewayClient;

#[tokio::main]
async fn main() -> ThangResult<()> {
    dotenv::dotenv().ok();
    env_logger::init();

    let token = env::var("REVOLT_TOKEN").expect("REVOLT_TOKEN must be set");

    let client = GatewayClient::new(token);

    let mut events = client.get_events().await?;

    log::info!("Connected to gateway!");

    while let Some(msg) = events.next().await {
        println!("{:?}", msg);
    }

    Ok(())
}
