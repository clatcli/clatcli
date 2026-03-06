use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

const DEFAULT_SYSTEM_PROMPT: &str = "\
You are clat, a shell command assistant. Convert the user's natural language request into a shell script.

If you need to reason or gather information about the system, use the available tools — do NOT put reasoning or explanation in your final response.

Output rules (strictly enforced):
- Respond with ONLY the shell commands, nothing else
- No backticks, no code fences, no markdown
- No preamble, no explanation, no trailing commentary
- Use bash syntax
- For multi-step tasks, chain commands with proper sequencing
- Prefer robust scripts; set -e is acceptable for simple tasks";

fn default_system_prompt() -> String {
    DEFAULT_SYSTEM_PROMPT.to_string()
}

fn default_api_url() -> String {
    "http://localhost:1234/v1".to_string()
}

fn default_model() -> String {
    "local-model".to_string()
}

fn default_true() -> bool {
    true
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

    /// Command names (first word of any script line) that skip the confirmation prompt
    #[serde(default)]
    pub auto_run_patterns: Vec<String>,

    /// Send MCP-style tool definitions with each request so the model can query
    /// the system (OS info, available commands) before writing the script
    #[serde(default = "default_true")]
    pub use_tools: bool,

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
            auto_run_patterns: vec![],
            use_tools: true,
            system_prompt: default_system_prompt(),
        }
    }
}

impl Config {
    /// Config lives in the same directory as the installed binary: ~/.clat/
    pub fn path() -> PathBuf {
        dirs::home_dir()
            .expect("cannot determine home directory")
            .join(".clat")
            .join("config.toml")
    }

    pub fn load() -> Result<Self> {
        let path = Self::path();
        if !path.exists() {
            let cfg = Self::default();
            cfg.save().ok(); // auto-create on first run; ignore errors (e.g. read-only fs)
            return Ok(cfg);
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
