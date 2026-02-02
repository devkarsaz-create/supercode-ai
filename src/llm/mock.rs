use crate::types::Message;
use crate::llm::Llm;
use async_trait::async_trait;

pub struct MockLlm {
    pub response: String,
}

impl MockLlm {
    pub fn new(response: impl Into<String>) -> Self {
        Self { response: response.into() }
    }
}

#[async_trait]
impl Llm for MockLlm {
    async fn chat(&self, _messages: &[Message]) -> anyhow::Result<String> {
        Ok(self.response.clone())
    }
}
