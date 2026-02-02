use crate::types::Message;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct LlamaClient {
    client: Client,
    pub endpoint: String,
    pub model: String,
}

#[derive(Error, Debug)]
pub enum LlmError {
    #[error("http error: {0}")]
    Http(#[from] reqwest::Error),
    #[error("invalid response")]
    InvalidResponse,
}

#[derive(Serialize)]
struct ChatRequest<'a> {
    model: &'a str,
    messages: &'a [LLMsg<'a>],
}

#[derive(Serialize)]
struct LLMsg<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Option<RespMsg>,
}

#[derive(Deserialize)]
struct RespMsg {
    content: Option<String>,
}

impl LlamaClient {
    pub fn new(endpoint: impl Into<String>, model: impl Into<String>) -> Self {
        Self {
            client: Client::new(),
            endpoint: endpoint.into(),
            model: model.into(),
        }
    }

    pub async fn request(&self, messages: &[Message]) -> Result<String, LlmError> {
        let msgs: Vec<LLMsg<'_>> = messages
            .iter()
            .map(|m| LLMsg {
                role: &m.role,
                content: &m.content,
            })
            .collect();

        let body = ChatRequest {
            model: &self.model,
            messages: &msgs,
        };

        let url = format!("{}/v1/chat/completions", self.endpoint.trim_end_matches('/'));

        let resp = self
            .client
            .post(&url)
            .json(&body)
            .send()
            .await?
            .json::<ChatResponse>()
            .await
            .map_err(LlmError::Http)?;

        let text = resp
            .choices
            .get(0)
            .and_then(|c| c.message.as_ref())
            .and_then(|m| m.content.clone())
            .ok_or(LlmError::InvalidResponse)?;

        Ok(text)
    }
}

// Implement the Llm trait for LlamaClient
use crate::llm::Llm;

#[async_trait::async_trait]
impl Llm for LlamaClient {
    async fn chat(&self, messages: &[Message]) -> anyhow::Result<String> {
        match self.request(messages).await {
            Ok(s) => Ok(s),
            Err(e) => Err(anyhow::anyhow!(e.to_string())),
        }
    }
}
