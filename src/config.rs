use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThemeName {
    DarkPlus,
    Light,
    Monokai,
    SolarizedDark,
    SolarizedLight,
    Dracula,
    OneDark,
    Nord,
    Gruvbox,
    Peacocks,
}

impl Default for ThemeName {
    fn default() -> Self {
        ThemeName::DarkPlus
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RuntimeConfig {
    pub llm_endpoint: String,
    pub llm_model: String,
    pub theme: ThemeName,
    pub model_dir: std::path::PathBuf,
    pub model_server_addr: std::net::SocketAddr,
}

impl Default for RuntimeConfig {
    fn default() -> Self {
        let mut model_dir = dirs::data_dir().unwrap_or_else(|| std::path::PathBuf::from("./"));
        model_dir.push("super-agent/models");
        Self {
            llm_endpoint: "http://127.0.0.1:8080".into(),
            llm_model: "local.gguf".into(),
            theme: ThemeName::default(),
            model_dir,
            model_server_addr: std::net::SocketAddr::from(([127,0,0,1], 11400)),
        }
    }
}

impl RuntimeConfig {
    pub fn path() -> Option<PathBuf> {
        if let Some(mut d) = dirs::config_dir() {
            d.push("super-agent");
            fs::create_dir_all(&d).ok()?;
            d.push("config.toml");
            Some(d)
        } else {
            None
        }
    }

    pub fn load() -> Self {
        if let Some(p) = Self::path() {
            if p.exists() {
                if let Ok(s) = fs::read_to_string(&p) {
                    if let Ok(cfg) = toml::from_str::<RuntimeConfig>(&s) {
                        return cfg;
                    }
                }
            }
        }
        RuntimeConfig::default()
    }

    pub fn save(&self) -> anyhow::Result<()> {
        if let Some(p) = Self::path() {
            let s = toml::to_string_pretty(self)?;
            fs::write(&p, s)?;
            return Ok(());
        }
        Err(anyhow::anyhow!("no config path"))
    }
}
