use crate::llm::Llm;
use crate::memory::store::MemoryStore;
use crate::tools::registry::ToolRegistry;
use crate::types::{AgentInput, AgentOutput, AgentId, Message};
use std::sync::Arc;

#[derive(Clone)]
pub struct SubAgent {
    pub id: AgentId,
    pub role: String,
    pub memory: MemoryStore,
    pub tools: ToolRegistry,
    pub llm: Arc<dyn Llm>,
}

impl SubAgent {
    pub fn new(role: impl Into<String>, llm: Arc<dyn Llm>) -> Self {
        Self {
            id: crate::types::new_id(),
            role: role.into(),
            memory: MemoryStore::new(),
            tools: ToolRegistry::new(),
            llm,
        }
    }

    pub async fn plan(&self, goal: &str) -> anyhow::Result<String> {
        let msg = Message::new("user", format!("Plan for goal: {}", goal));
        let ctx = self.memory.get_short();
        // add short term memory for context
        // call LLM for a plan
        let mut messages = ctx;
        messages.push(msg.clone());
        let resp = self.llm.chat(&messages).await?;
        self.memory.add_short(Message::new("assistant", &resp));
        Ok(resp)
    }

    pub async fn execute(&self, plan: &str) -> anyhow::Result<AgentOutput> {
        // a very small deterministic executor that calls tools / microagents
        let input = AgentInput {
            text: plan.to_string(),
        };

        // for v0.1 try the echo tool if present
        match self.tools.run("echo", input) {
            Ok(to) => Ok(AgentOutput { text: to.text }),
            Err(_) => Ok(AgentOutput {
                text: format!("executed: {}", plan),
            }),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::llm::mock::MockLlm;
    use std::sync::Arc;

    #[tokio::test]
    async fn test_plan_and_execute_with_mock() {
        let mock = Arc::new(MockLlm::new("this is a plan"));
        let agent = SubAgent::new("planner", mock);

        let plan = agent.plan("do something").await.expect("plan failed");
        assert_eq!(plan, "this is a plan");

        // register a local echo tool and execute
        agent.tools.register(Arc::new(crate::tools::registry::EchoTool));
        let out = agent.execute(&plan).await.expect("exec failed");
        assert_eq!(out.text, "this is a plan");
    }
}
