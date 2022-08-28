use std::error::Error;

pub type ThangResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub struct Context {
    pub http: twilight_http::Client,
    pub bridge_webhook_url: String,
    pub bridge_webhook_id: Id<WebhookMarker>,
    pub bridge_webhook_token: String,
    pub bridge_channel_id: Id<ChannelMarker>,
    pub eludris_gateway_url: String,
    pub eludris_rest_url: String,
    pub eludris_http_client: reqwest::Client,
    pub eludris_ws_writer: WsWriter,
}
