use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_SYSTEM_PROMPT: &str = "\
You are clat, a shell command assistant. Convert the user's natural language request into a shell script.
Rules:
- Output ONLY shell commands/script, nothing else
- No markdown code fences, no explanations, no preamble
- Use bash syntax
- For multi-step tasks, chain commands with proper sequencing and error handling
- Prefer robust scripts (set -e is fine for simple tasks)";

fn default_system_prompt() -> String {
    DEFAULT_SYSTEM_PROMPT.to_string()
}

fn default_api_url() -> String {
    "http://localhost:1234/v1".to_string()
}

fn default_model() -> String {
    "local-model".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    #[serde(default = "default_api_url")]
    pub api_url: String,

    #[serde(default = "default_model")]
    pub model: String,

    #[serde(default)]
    pub api_key: String,

    /// Skip confirmation prompt and execute immediately
    #[serde(default)]
    pub auto_run: bool,

    #[serde(default = "default_system_prompt")]
    pub system_prompt: String,
}

impl Default for Config {
    fn default() -> Self {
        Config {
            api_url: default_api_url(),
            model: default_model(),
            api_key: String::new(),
            auto_run: false,
            system_prompt: default_system_prompt(),
        }
    }
}

impl Config {
    pub fn path() -> PathBuf {
        dirs::config_dir()
            .unwrap_or_else(|| dirs::home_dir().unwrap().join(".config"))
            .join("clat")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let content = std::fs::read_to_string(&path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    pub fn save(&self) -> Result<()> {
        let path = Self::path();
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }
        let content = toml::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }
}
