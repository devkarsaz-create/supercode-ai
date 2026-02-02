use crate::types::AgentInput;
use serde::{Deserialize, Serialize};

pub type ToolResult = anyhow::Result<ToolOutput>;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ToolOutput {
    pub text: String,
}

#[derive(Debug)]
pub struct ToolInfo {
    pub name: String,
    pub description: String,
}

pub trait Tool: Send + Sync {
    fn name(&self) -> &'static str;
    fn description(&self) -> &'static str;
    fn run(&self, input: AgentInput) -> ToolResult;
}

use parking_lot::RwLock;
use std::collections::HashMap;
use std::sync::Arc;

#[derive(Default, Clone)]
pub struct ToolRegistry {
    inner: Arc<RwLock<HashMap<String, Arc<dyn Tool>>>>,
}

impl ToolRegistry {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub fn register(&self, t: Arc<dyn Tool>) {
        self.inner
            .write()
            .insert(t.name().to_string(), Arc::clone(&t));
    }

    pub fn run(&self, name: &str, input: AgentInput) -> ToolResult {
        let map = self.inner.read();
        let t = map.get(name).ok_or_else(|| anyhow::anyhow!("tool not found"))?;
        t.run(input)
    }
}

// Example tool
pub struct EchoTool;
impl Tool for EchoTool {
    fn name(&self) -> &'static str {
        "echo"
    }

    fn description(&self) -> &'static str {
        "Returns the input text"
    }

    fn run(&self, input: AgentInput) -> ToolResult {
        Ok(ToolOutput { text: input.text })
    }
}
