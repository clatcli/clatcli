use anyhow::{anyhow, Result};
use colored::Colorize;
use serde::{Deserialize, Serialize};
use std::io::Write;

use crate::config::Config;
use crate::tools;

// ── Message types ──────────────────────────────────────────────────────────────

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Message {
    pub role: String,

    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,

    /// Set on assistant messages when the model wants to call tools.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_calls: Option<Vec<ToolCall>>,

    /// Set on tool-result messages.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
}

impl Message {
    fn system(content: impl Into<String>) -> Self {
        Self { role: "system".into(), content: Some(content.into()), tool_calls: None, tool_call_id: None }
    }
    fn user(content: impl Into<String>) -> Self {
        Self { role: "user".into(), content: Some(content.into()), tool_calls: None, tool_call_id: None }
    }
    fn tool_result(id: impl Into<String>, content: impl Into<String>) -> Self {
        Self { role: "tool".into(), content: Some(content.into()), tool_calls: None, tool_call_id: Some(id.into()) }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub function: ToolCallFn,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ToolCallFn {
    pub name: String,
    pub arguments: String,
}

// ── Request / Response ─────────────────────────────────────────────────────────

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
    #[serde(skip_serializing_if = "Option::is_none")]
    tools: Option<Vec<serde_json::Value>>,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
    finish_reason: Option<String>,
}

// ── Model listing ──────────────────────────────────────────────────────────────

#[derive(Deserialize)]
struct ModelsResponse {
    data: Vec<ModelEntry>,
}

#[derive(Deserialize)]
pub struct ModelEntry {
    pub id: String,
    /// LM Studio includes a "state" field ("not loaded" / "loaded")
    pub state: Option<String>,
}

// ── Helpers ────────────────────────────────────────────────────────────────────

fn make_client() -> Result<reqwest::blocking::Client> {
    Ok(reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(120))
        .build()?)
}

fn base(config: &Config) -> String {
    config.api_url.trim_end_matches('/').to_string()
}

fn authed(
    config: &Config,
    builder: reqwest::blocking::RequestBuilder,
) -> reqwest::blocking::RequestBuilder {
    if config.api_key.is_empty() {
        builder
    } else {
        builder.bearer_auth(&config.api_key)
    }
}

fn connect_err(config: &Config, e: reqwest::Error) -> anyhow::Error {
    if e.is_connect() || e.is_timeout() {
        anyhow!(
            "Could not connect to {}.\nIs LM Studio (or your inference server) running?",
            config.api_url
        )
    } else {
        anyhow!("{e}")
    }
}

/// Remove all <think>…</think> blocks emitted by reasoning models (DeepSeek-R1,
/// QwQ, etc.) before extracting the script.
fn strip_think_blocks(s: &str) -> String {
    let mut out = s.to_string();
    loop {
        match (out.find("<think>"), out.find("</think>")) {
            (Some(start), Some(end)) if start < end => {
                out = format!("{}{}", &out[..start], &out[end + "</think>".len()..]);
            }
            _ => break,
        }
    }
    out
}

/// Extract a script from the model's response, handling the common ways models
/// misbehave despite being asked for raw shell output:
///
/// 1. `<think>…</think>` blocks (reasoning models: DeepSeek-R1, QwQ, …)
/// 2. A markdown code fence anywhere in the response — including after preamble
///    text like "Here's the script:" that the current-line check would miss
/// 3. Clean output — returned as-is
fn clean_response(raw: &str) -> String {
    // Pass 1: remove all reasoning blocks
    let s = strip_think_blocks(raw.trim());
    let s = s.trim();

    // Pass 2: if a code fence exists anywhere, extract just its contents.
    // This handles both "```\nscript\n```" at the start *and*
    // "Some explanation:\n```bash\nscript\n```" with leading prose.
    if let Some(open) = s.find("```") {
        let rest = &s[open + 3..];
        // Skip the optional language tag line (bash, sh, zsh, shell, …)
        let body = match rest.find('\n') {
            Some(nl) => &rest[nl + 1..],
            None => rest,
        };
        if let Some(close) = body.find("```") {
            return body[..close].trim().to_string();
        }
    }

    // Pass 3: model followed instructions — return trimmed text directly
    s.to_string()
}

// ── Public functions ───────────────────────────────────────────────────────────

/// Generate a shell script from a natural language prompt.
/// If `config.use_tools` is true, the model may call tools to inspect the
/// system before producing its final answer.
pub fn generate_script(config: &Config, prompt: &str) -> Result<String> {
    let client = make_client()?;
    let tool_defs = if config.use_tools { Some(tools::definitions()) } else { None };

    let mut messages = vec![
        Message::system(&config.system_prompt),
        Message::user(prompt),
    ];

    const MAX_ROUNDS: usize = 8;
    for _ in 0..MAX_ROUNDS {
        let request = ChatRequest {
            model: config.model.clone(),
            messages: messages.clone(),
            temperature: 0.1,
            tools: tool_defs.clone(),
        };

        let resp = authed(config, client.post(format!("{}/chat/completions", base(config))).json(&request))
            .send()
            .map_err(|e| connect_err(config, e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            return Err(anyhow!("API error {status}: {body}"));
        }

        let chat: ChatResponse = resp.json()?;
        let choice = chat.choices.into_iter().next()
            .ok_or_else(|| anyhow!("empty response from API"))?;

        match choice.finish_reason.as_deref() {
            Some("tool_calls") => {
                let calls = choice.message.tool_calls.clone().unwrap_or_default();
                messages.push(choice.message);

                for tc in &calls {
                    // Clear "thinking..." and show which tool is running
                    eprint!("\r\x1b[K");
                    eprint!("{} {}  ", "tool:".dimmed(), tc.function.name.cyan());
                    std::io::stderr().flush().ok();

                    let result = tools::dispatch(&tc.function.name, &tc.function.arguments);
                    messages.push(Message::tool_result(tc.id.clone(), result));
                }

                // Restore thinking indicator for next round
                eprint!("\r\x1b[K");
                eprint!("{}", "thinking...".dimmed());
                std::io::stderr().flush().ok();
            }
            _ => {
                eprint!("\r\x1b[K");
                let raw = choice.message.content.unwrap_or_default();
                return Ok(clean_response(&raw));
            }
        }
    }

    Err(anyhow!(
        "exceeded maximum tool-call rounds — the model may not support tool calling.\n\
         Fix: set use_tools = false in ~/.clat/config.toml"
    ))
}

/// List models available from the API.
pub fn list_models(config: &Config) -> Result<Vec<ModelEntry>> {
    let client = make_client()?;
    let resp = authed(config, client.get(format!("{}/models", base(config))))
        .send()
        .map_err(|e| connect_err(config, e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(anyhow!("API error {status}: {body}"));
    }

    let models: ModelsResponse = resp.json()?;
    Ok(models.data)
}

/// Request LM Studio to load a model. Derives the management API base URL by
/// stripping the `/v1` path from `config.api_url`.
/// This is LM Studio-specific; other APIs will return an error.
pub fn load_model(config: &Config, model_id: &str) -> Result<()> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(300)) // loading can be slow
        .build()?;

    // http://localhost:1234/v1  →  http://localhost:1234
    let mgmt_base = config.api_url
        .trim_end_matches('/')
        .trim_end_matches("/v1");

    let url = format!("{mgmt_base}/api/v0/models/load");
    let body = serde_json::json!({ "identifier": model_id });

    let resp = authed(config, client.post(&url).json(&body))
        .send()
        .map_err(|e| connect_err(config, e))?;

    if !resp.status().is_success() {
        let status = resp.status();
        let text = resp.text().unwrap_or_default();
        return Err(anyhow!(
            "Failed to load model (HTTP {status}): {text}\n\
             Note: model loading requires LM Studio with the management API enabled."
        ));
    }

    Ok(())
}
