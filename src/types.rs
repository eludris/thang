use std::{error::Error, sync::Arc};

use futures::stream::{SplitSink, SplitStream};
use serde::{Deserialize, Serialize};
use tokio::{net::TcpStream, sync::Mutex};
use tokio_tungstenite::{
    tungstenite::protocol::Message as TungstenMessage, MaybeTlsStream, WebSocketStream,
};
use twilight_model::id::{
    marker::{ChannelMarker, WebhookMarker},
    Id,
};

pub type ThangResult<T> = Result<T, Box<dyn Error + Send + Sync>>;
pub type WsWriter = SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, TungstenMessage>;
pub type WsReader = SplitStream<WebSocketStream<MaybeTlsStream<TcpStream>>>;
pub type ContextT = Arc<Context>;

pub struct Context {
    pub http: twilight_http::Client,
    pub bridge_webhook_id: Id<WebhookMarker>,
    pub bridge_webhook_token: String,
    pub bridge_channel_id: Id<ChannelMarker>,
    pub eludris_gateway_url: String,
    pub eludris_rest_url: String,
    pub eludris_http_client: reqwest::Client,
    pub eludris_ws_writer: Mutex<WsWriter>,
}

#[derive(Serialize, Deserialize)]
pub struct Message {
    pub author: String,
    pub content: String,
}
