use serde_json::{json, Value};
use std::process::Command;

/// Returns tool definitions in OpenAI function-calling format.
pub fn definitions() -> Vec<Value> {
    vec![
        json!({
            "type": "function",
            "function": {
                "name": "get_system_info",
                "description": "Returns OS, architecture, shell, current working directory, home directory, and current user. Call this when the script depends on system context.",
                "parameters": {
                    "type": "object",
                    "properties": {},
                    "required": []
                }
            }
        }),
        json!({
            "type": "function",
            "function": {
                "name": "check_commands",
                "description": "Checks whether CLI commands are available on PATH. Use this before writing a script that depends on a particular tool being installed.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "commands": {
                            "type": "array",
                            "items": { "type": "string" },
                            "description": "Command names to check, e.g. [\"docker\", \"brew\", \"git\"]"
                        }
                    },
                    "required": ["commands"]
                }
            }
        }),
    ]
}

/// Execute a tool by name and return its result as a JSON string.
pub fn dispatch(name: &str, arguments: &str) -> String {
    match name {
        "get_system_info" => get_system_info(),
        "check_commands" => check_commands(arguments),
        _ => json!({ "error": format!("unknown tool: {name}") }).to_string(),
    }
}

fn get_system_info() -> String {
    json!({
        "os":   std::env::consts::OS,
        "arch": std::env::consts::ARCH,
        "shell": std::env::var("SHELL").unwrap_or_else(|_| "unknown".into()),
        "cwd":  std::env::current_dir()
                    .map(|p| p.display().to_string())
                    .unwrap_or_else(|_| "unknown".into()),
        "home": std::env::var("HOME").unwrap_or_else(|_| "unknown".into()),
        "user": whoami::username(),
    })
    .to_string()
}

fn check_commands(arguments: &str) -> String {
    let args: Value = serde_json::from_str(arguments).unwrap_or(Value::Null);
    let commands = match args["commands"].as_array() {
        Some(v) => v.clone(),
        None => return json!({ "error": "missing 'commands' array" }).to_string(),
    };

    let mut result = serde_json::Map::new();
    for cmd in &commands {
        if let Some(name) = cmd.as_str() {
            let available = Command::new("which")
                .arg(name)
                .output()
                .map(|o| o.status.success())
                .unwrap_or(false);
            result.insert(name.to_string(), Value::Bool(available));
        }
    }
    Value::Object(result).to_string()
}
