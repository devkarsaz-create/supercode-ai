use crate::types::{AgentInput, AgentOutput};

pub trait MicroAgent: Send + Sync {
    fn name(&self) -> &'static str;
    fn execute(&self, input: AgentInput) -> anyhow::Result<AgentOutput>;
}

pub struct UppercaseAgent;

impl MicroAgent for UppercaseAgent {
    fn name(&self) -> &'static str {
        "uppercase"
    }

    fn execute(&self, input: AgentInput) -> anyhow::Result<AgentOutput> {
        Ok(AgentOutput {
            text: input.text.to_uppercase(),
        })
    }
}
