use crate::models::manager::ModelManager;
use crate::types::Message;
use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProviderKind {
    Auto,
    LlamaCpp,
    Ollama,
    Mock,
}

#[async_trait]
pub trait Provider: Send + Sync {
    fn name(&self) -> &'static str;
    async fn start(&self) -> anyhow::Result<()> {
        Ok(())
    }
    async fn stop(&self) -> anyhow::Result<()> {
        Ok(())
    }
    async fn is_running(&self) -> bool {
        true
    }
    async fn chat(&self, messages: &[Message]) -> anyhow::Result<String>;
}

pub struct MockProvider {
    pub model: PathBuf,
}

#[async_trait]
impl Provider for MockProvider {
    fn name(&self) -> &'static str {
        "mock"
    }

    async fn chat(&self, messages: &[Message]) -> anyhow::Result<String> {
        // Very cheap deterministic response that includes model name and last user message
        let last = messages.last().map(|m| m.content.clone()).unwrap_or_default();
        Ok(format!("[mock:{}] echo: {}", self.model.file_name().and_then(|s| s.to_str()).unwrap_or("m"), last))
    }
}

pub struct LlamaProvider {
    pub binary: Option<PathBuf>,
    pub model: PathBuf,
    pub addr: std::net::SocketAddr,
    child: tokio::sync::Mutex<Option<tokio::process::Child>>,
}

impl LlamaProvider {
    pub fn new(binary: Option<PathBuf>, model: PathBuf, addr: std::net::SocketAddr) -> Self {
        Self { binary, model, addr, child: tokio::sync::Mutex::new(None) }
    }

    fn find_binary(&self) -> Option<PathBuf> {
        if let Some(b) = &self.binary { return Some(b.clone()); }
        // try PATH
        if let Ok(path) = which::which("server") {
            return Some(path);
        }
        if let Ok(path) = which::which("llama") {
            return Some(path);
        }
        None
    }
}

#[async_trait]
impl Provider for LlamaProvider {
    fn name(&self) -> &'static str { "llama" }

    async fn start(&self) -> anyhow::Result<()> {
        let bin = self.find_binary().ok_or_else(|| anyhow::anyhow!("llama binary not found"))?;
        let mut child = tokio::process::Command::new(bin)
            .arg("--model")
            .arg(self.model.as_path())
            .arg("--http")
            .arg(format!("{}", self.addr.port()))
            .spawn()?;
        *self.child.lock().await = Some(child);
        // wait for health endpoint with exponential backoff (try up to ~12 times)
        let url = format!("http://{}:{}/v1/health", self.addr.ip(), self.addr.port());
        let mut delay_ms = 200u64;
        for _ in 0..12 {
            match reqwest::get(&url).await {
                Ok(resp) => {
                    if resp.status().is_success() {
                        return Ok(());
                    }
                }
                Err(_) => {}
            }
            tokio::time::sleep(std::time::Duration::from_millis(delay_ms)).await;
            delay_ms = (delay_ms.saturating_mul(2)).min(3000);
        }
        Err(anyhow::anyhow!("llama provider failed to start or health-check failed"))
    }

    async fn stop(&self) -> anyhow::Result<()> {
        if let Some(mut c) = self.child.lock().await.take() {
            c.kill().await.ok();
        }
        Ok(())
    }

    async fn is_running(&self) -> bool {
        if let Some(c) = &*self.child.lock().await {
            c.id().is_some()
        } else { false }
    }

    async fn chat(&self, messages: &[Message]) -> anyhow::Result<String> {
        // proxy to local llama.cpp http endpoint
        let url = format!("http://{}/v1/chat/completions", self.addr);
        #[derive(Serialize)] struct Req<'a> { model: &'a str, messages: &'a [crate::types::Message] }
        let model_name = self.model.to_str().unwrap_or("");
        let body = Req { model: model_name, messages };
        let resp = reqwest::Client::new().post(&url).json(&body).send().await?.json::<serde_json::Value>().await?;
        // extract text similar to LlamaClient
        let text = resp
            .get("choices")
            .and_then(|c| c.get(0))
            .and_then(|ch| ch.get("message"))
            .and_then(|m| m.get("content"))
            .and_then(|v| v.as_str())
            .map(|s| s.to_string())
            .unwrap_or_else(|| "".to_string());
        Ok(text)
    }
}

pub struct ModelServer {
    manager: Arc<ModelManager>,
    providers: Arc<RwLock<HashMap<String, Arc<dyn Provider>>>>,
    pub addr: SocketAddr,
}

impl ModelServer {
    pub fn new(manager: Arc<ModelManager>, bind: SocketAddr) -> Self {
        Self { manager, providers: Arc::new(RwLock::new(HashMap::new())), addr: bind }
    }

    pub async fn start_local_server(&self) -> anyhow::Result<()> {
        // spawn a minimal axum server that serves /v1/models and /v1/chat/completions
        use axum::{routing::{get, post}, Router, extract::Json, response::IntoResponse};
        use serde_json::json;

        let mgr = Arc::clone(&self.manager);
        let providers = Arc::clone(&self.providers);

        #[derive(Deserialize)]
        struct ChatReq {
            model: Option<String>,
            messages: Vec<Message>,
        }

        #[derive(Serialize)]
        struct ChatChoice {
            message: Option<MessageResp>,
        }
        #[derive(Serialize)]
        struct MessageResp {
            content: Option<String>,
        }
        #[derive(Serialize)]
        struct ChatResp {
            choices: Vec<ChatChoice>,
        }

        let list_models = move || {
            let mgr = Arc::clone(&mgr);
            async move {
                match mgr.discover() {
                    Ok(ms) => {
                        let models: Vec<_> = ms.into_iter().map(|m| json!({"name": m.name, "format": m.format, "size": m.size})).collect();
                        (axum::http::StatusCode::OK, Json(json!({"models": models})))
                    }
                    Err(e) => (axum::http::StatusCode::INTERNAL_SERVER_ERROR, Json(json!({"error": e.to_string()}))),
                }
            }
        };

        let chat = move |Json(payload): Json<ChatReq>| {
            let providers = Arc::clone(&providers);
            async move {
                // select provider: if model supplied and provider registered, else use mock provider if any
                let model = payload.model.clone();
                let mut resp_text = String::from("no provider");
                if let Some(mn) = model {
                    let pmap = providers.read().await;
                    if let Some(p) = pmap.get(&mn) {
                        match p.chat(&payload.messages).await {
                            Ok(s) => resp_text = s,
                            Err(e) => resp_text = format!("error: {}", e),
                        }
                    } else {
                        resp_text = format!("no provider registered for model {}", mn);
                    }
                } else {
                    // try any provider
                    let pmap = providers.read().await;
                    if let Some((_k, p)) = pmap.iter().next() {
                        match p.chat(&payload.messages).await {
                            Ok(s) => resp_text = s,
                            Err(e) => resp_text = format!("error: {}", e),
                        }
                    } else {
                        resp_text = "no providers available".to_string();
                    }
                }
                let choice = ChatChoice { message: Some(MessageResp { content: Some(resp_text) }) };
                let out = ChatResp { choices: vec![choice] };
                (axum::http::StatusCode::OK, Json(out))
            }
        };

        let app = Router::new().route("/v1/models", get(list_models)).route("/v1/chat/completions", post(chat));
        let addr = self.addr;
        let listener = tokio::net::TcpListener::bind(addr).await?;
        let server = axum::serve(listener, app.into_make_service());
        tokio::spawn(async move {
            if let Err(e) = server.await {
                tracing::error!("model server error: {}", e);
            }
        });
        Ok(())
    }

    pub async fn register_mock_for_model(&self, model_name: &str) -> anyhow::Result<()> {
        let ms = self.manager.discover()?;
        for m in ms {
            if m.name == model_name {
                let p: Arc<dyn Provider> = Arc::new(MockProvider { model: m.path.clone() });
                self.providers.write().await.insert(model_name.to_string(), p);
                return Ok(());
            }
        }
        Err(anyhow::anyhow!("model not found"))
    }

    pub async fn register_provider(&self, name: &str, provider: Arc<dyn Provider>) -> anyhow::Result<()> {
        self.providers.write().await.insert(name.to_string(), provider);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_register_mock_for_model() -> anyhow::Result<()> {
        let td = tempdir()?;
        let sample = td.path().join("mymodel.gguf");
        std::fs::write(&sample, b"dummy")?;
        let mgr = Arc::new(ModelManager { dir: td.path().to_path_buf() });
        let addr: std::net::SocketAddr = "127.0.0.1:11401".parse().unwrap();
        let server = ModelServer::new(mgr, addr);
        server.register_mock_for_model("mymodel").await?;
        let map = server.providers.read().await;
        assert!(map.contains_key("mymodel"));
        Ok(())
    }
}
