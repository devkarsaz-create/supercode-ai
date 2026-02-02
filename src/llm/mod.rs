pub mod llama;
pub mod mock;

use crate::types::Message;
use async_trait::async_trait;

#[async_trait]
pub trait Llm: Send + Sync {
    async fn chat(&self, messages: &[Message]) -> anyhow::Result<String>;
}