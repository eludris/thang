use std::error::Error;

pub type ThangResult<T> = Result<T, Box<dyn Error + Send + Sync>>;

pub struct Context {
    pub http: twilight_http::Client,
    pub webhook_url: String,
    pub webhook_id: u64,
    pub bridge_channel_id: u64,
    pub eludris_gateway_url: String,
    pub eludris_rest_url: String,
    pub eludris_http_client: reqwest::Client,
}
