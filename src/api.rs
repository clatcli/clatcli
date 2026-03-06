use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::config::Config;

#[derive(Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    temperature: f32,
}

#[derive(Serialize, Deserialize)]
struct Message {
    role: String,
    content: String,
}

#[derive(Deserialize)]
struct ChatResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: Message,
}

pub fn generate_script(config: &Config, prompt: &str) -> Result<String> {
    let client = reqwest::blocking::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?;

    let request = ChatRequest {
        model: config.model.clone(),
        messages: vec![
            Message {
                role: "system".to_string(),
                content: config.system_prompt.clone(),
            },
            Message {
                role: "user".to_string(),
                content: prompt.to_string(),
            },
        ],
        temperature: 0.1,
    };

    let mut req_builder = client
        .post(format!("{}/chat/completions", config.api_url.trim_end_matches('/')))
        .json(&request);

    if !config.api_key.is_empty() {
        req_builder = req_builder.bearer_auth(&config.api_key);
    }

    let resp = req_builder.send().map_err(|e| {
        if e.is_connect() {
            anyhow!(
                "Could not connect to API at {}. Is LM Studio (or your inference server) running?",
                config.api_url
            )
        } else {
            anyhow!("Request failed: {}", e)
        }
    })?;

    if !resp.status().is_success() {
        let status = resp.status();
        let body = resp.text().unwrap_or_default();
        return Err(anyhow!("API error {}: {}", status, body));
    }

    let chat: ChatResponse = resp.json()?;
    let raw = chat
        .choices
        .into_iter()
        .next()
        .map(|c| c.message.content)
        .unwrap_or_default();

    Ok(strip_code_fences(raw.trim()))
}

/// Strip markdown code fences if the model wrapped its output anyway.
fn strip_code_fences(s: &str) -> String {
    if s.starts_with("```") {
        let mut lines: Vec<&str> = s.lines().collect();
        // Remove opening fence (```bash, ```sh, ``` etc.)
        if !lines.is_empty() && lines[0].starts_with("```") {
            lines.remove(0);
        }
        // Remove closing fence
        if lines.last() == Some(&"```") {
            lines.pop();
        }
        return lines.join("\n");
    }
    s.to_string()
}
