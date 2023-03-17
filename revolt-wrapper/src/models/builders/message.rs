use crate::{
    models::{Masquerade, Message},
    HttpClient,
};

use models::Result;
use serde::Serialize;

use crate::models::File;

#[derive(Debug, Serialize)]
pub struct SendMessage<'a> {
    #[serde(skip)]
    http: &'a HttpClient,
    target: &'a str,
    content: Option<String>,
    attachments: Option<Vec<File>>,
    replies: Option<Vec<String>>,
    masquerade: Option<Masquerade>,
}

impl<'a> SendMessage<'a> {
    pub(crate) fn new(http: &'a HttpClient, target: &'a str) -> Self {
        Self {
            http,
            target,
            content: None,
            attachments: None,
            replies: None,
            masquerade: None,
        }
    }

    pub fn content(mut self, content: String) -> Self {
        self.content = Some(content);
        self
    }

    pub fn attachments(mut self, attachments: Vec<File>) -> Self {
        self.attachments = Some(attachments);
        self
    }

    pub fn replies(mut self, replies: Vec<String>) -> Self {
        self.replies = Some(replies);
        self
    }

    pub fn masquerade(mut self, masquerade: Masquerade) -> Self {
        self.masquerade = Some(masquerade);
        self
    }

    pub async fn send(self) -> Result<Message> {
        let url = format!("{}/channels/{}/messages", self.http.rest_url, self.target);
        Ok(self
            .http
            .client
            .post(&url)
            .header("x-bot-token", &self.http.token)
            .json(&self)
            .send()
            .await?
            .json()
            .await?)
    }
}
