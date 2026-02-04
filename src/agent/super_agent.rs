use crate::agent::sub_agent::SubAgent;
use crate::graph::dag::AgentGraph;
use crate::llm::llama::LlamaClient;
use crate::llm::Llm;
use crate::tools::registry::EchoTool;
use crate::types::AgentState;
use std::sync::Arc;

pub struct SuperAgent {
    pub graph: AgentGraph,
    pub scheduler: AgentState,
    pub llm: Arc<dyn Llm>,
}

impl SuperAgent {
    pub fn new() -> Self {
        let endpoint = std::env::var("LLAMA_ENDPOINT").unwrap_or_else(|_| "http://127.0.0.1:8080".into());
        let model = std::env::var("LLAMA_MODEL").unwrap_or_else(|_| "local.gguf".into());
        let llm: Arc<dyn Llm> = Arc::new(LlamaClient::new(endpoint, model));
        Self {
            graph: AgentGraph::new(),
            scheduler: AgentState::Idle,
            llm,
        }
    }

    pub async fn run_goal(&mut self, goal: String) -> anyhow::Result<()> {
        self.scheduler = AgentState::Planning;

        // create planner subagent
        let planner = SubAgent::new("planner", Arc::clone(&self.llm));
        // register a basic echo tool so execution can be demonstrated
        let reg = &planner.tools;
        reg.register(Arc::new(EchoTool));

        let plan = planner.plan(&goal).await?;
        self.graph.add_node("planner", AgentState::Planning);
        self.scheduler = AgentState::Executing;

        let executor = SubAgent::new("executor", Arc::clone(&self.llm));
        executor.tools.register(Arc::new(EchoTool));
        let out = executor.execute(&plan).await?;
        self.graph.add_node("executor", AgentState::Executing);
        self.graph.add_edge(0, 1);

        self.scheduler = AgentState::Reviewing;

        // simple critic via llm
        let critic = SubAgent::new("critic", Arc::clone(&self.llm));
        let critique = match critic.plan(&out.text).await {
            Ok(text) => text,
            Err(e) => format!("critic error: {}", e),
        };
        self.graph.add_node("critic", AgentState::Reviewing);
        self.graph.add_edge(1, 2);

        self.scheduler = AgentState::Completed;

        println!("Plan:\n{}\n\nExecution:\n{}\n\nReview:\n{}", plan, out.text, critique);
        Ok(())
    }
}
