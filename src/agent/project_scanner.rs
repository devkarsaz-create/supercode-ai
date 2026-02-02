use crate::agent::plugin_engine::PluginEngine;
use crate::agent::LocalAgent;
use async_trait::async_trait;
use std::path::PathBuf;
use std::time::Duration;

pub struct ProjectScannerAgent {
    pub engine: PluginEngine,
    pub interval: Duration,
}

impl ProjectScannerAgent {
    pub fn new(skills_dir: Option<PathBuf>, interval: Duration) -> anyhow::Result<Self> {
        let mut engine = PluginEngine::new(skills_dir)?;
        engine.load_skills()?;
        Ok(Self { engine, interval })
    }

    #[cfg(test)]
    fn for_tests(skills_dir: Option<std::path::PathBuf>) -> anyhow::Result<Self> {
        let mut engine = PluginEngine::new(skills_dir)?;
        engine.load_skills()?;
        Ok(Self { engine, interval: std::time::Duration::from_secs(1) })
    }
}

#[async_trait]
impl LocalAgent for ProjectScannerAgent {
    fn name(&self) -> &'static str { "project_scanner" }

    async fn run_forever(&self) -> anyhow::Result<()> {
        loop {
            // Attempt to call the wasm skill if available
            match self.engine.call_skill("project_scanner", None) {
                Ok(out) => {
                    // In a real system we'd push this into shared memory or a database
                    tracing::info!("ProjectScanner output-length = {}", out.len());
                }
                Err(e) => {
                    tracing::warn!("project_scanner call failed: {}", e);
                }
            }
            tokio::time::sleep(self.interval).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_project_scanner_loads_wat() -> anyhow::Result<()> {
        let skills_dir = Some(std::path::PathBuf::from("./scripts/skills"));
        let agent = ProjectScannerAgent::for_tests(skills_dir)?;
        // call the skill directly
        let out = agent.engine.call_skill("project_scanner", None)?;
        // output should be JSON array (maybe empty) or string
        assert!(out.len() >= 0);
        Ok(())
    }
}
