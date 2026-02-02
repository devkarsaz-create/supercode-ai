pub mod super_agent;
pub mod sub_agent;
pub mod micro_agent;
pub mod plugin_engine;
pub mod project_scanner;

use async_trait::async_trait;

#[async_trait]
pub trait LocalAgent: Send + Sync {
    fn name(&self) -> &'static str;
    async fn run_forever(&self) -> anyhow::Result<()>;
}

// Export a helper to spawn a LocalAgent to background
pub fn spawn_agent<A: LocalAgent + 'static>(agent: A) {
    tokio::spawn(async move {
        if let Err(e) = agent.run_forever().await {
            tracing::error!("agent {} exited with error: {}", agent.name(), e);
        }
    });
}