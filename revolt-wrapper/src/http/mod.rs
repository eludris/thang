use crate::models::{Member, SendMessage, User};
use models::Result;
use reqwest::Client;

const REST_URL: &str = "https://api.revolt.chat/";

#[derive(Debug)]
pub struct HttpClient {
    pub(crate) client: Client,
    pub(crate) rest_url: String,
    pub(crate) token: String,
}

impl HttpClient {
    pub fn new(token: String) -> Self {
        Self {
            client: Client::new(),
            rest_url: REST_URL.to_string(),
            token,
        }
    }

    pub fn rest_url(mut self, rest_url: String) -> Self {
        self.rest_url = rest_url;
        self
    }

    pub fn send_message<'a>(&'a self, target: &'a str) -> SendMessage<'a> {
        SendMessage::new(self, target)
    }

    pub async fn fetch_self(&self) -> Result<User> {
        let url = format!("{}/users/@me", self.rest_url);
        Ok(self
            .client
            .get(&url)
            .header("x-bot-token", &self.token)
            .send()
            .await?
            .json()
            .await?)
    }

    pub async fn fetch_member(&self, target: &str, member: &str) -> Result<Member> {
        let url = format!("{}/servers/{}/members/{}", self.rest_url, target, member);
        Ok(self
            .client
            .get(&url)
            .header("x-bot-token", &self.token)
            .send()
            .await?
            .json()
            .await?)
    }
}
