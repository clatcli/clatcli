use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod api;
mod config;
mod executor;
mod tools;

#[derive(Parser)]
#[command(
    name = "clat",
    about = "Natural language shell assistant — describe what you want, get a script",
    long_about = None,
)]
struct Cli {
    /// What you want to do (in plain English)
    #[arg(trailing_var_arg = true, required = false)]
    prompt: Vec<String>,

    /// Skip confirmation and execute immediately
    #[arg(short = 'y', long)]
    yes: bool,

    /// Show generated script but don't execute
    #[arg(short = 'n', long = "dry-run")]
    dry_run: bool,

    /// Override the model from config
    #[arg(long)]
    model: Option<String>,

    /// Override the API URL from config
    #[arg(long)]
    api: Option<String>,

    /// Verbose: show prompt, API URL, and model before calling
    #[arg(short, long)]
    verbose: bool,

    /// Show config path and current settings
    #[arg(long)]
    config: bool,

    /// Write default config file to ~/.clat/config.toml (won't overwrite)
    #[arg(long)]
    init: bool,

    /// List models available from the API
    #[arg(short = 'l', long = "list")]
    list_models: bool,

    /// Load a model in LM Studio before running (LM Studio only)
    #[arg(short = 'L', long = "load", value_name = "MODEL_ID")]
    load_model: Option<String>,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut cfg = config::Config::load()?;

    // --init
    if cli.init {
        let path = config::Config::path();
        if path.exists() {
            println!("Config already exists at: {}", path.display());
        } else {
            cfg.save()?;
            println!("Created config at: {}", path.display());
            println!("Edit it to set your API URL, model, and options.");
        }
        return Ok(());
    }

    // --config
    if cli.config {
        let path = config::Config::path();
        println!("{} {}", "Config:".dimmed(), path.display());
        println!("{}", toml::to_string_pretty(&cfg)?);
        return Ok(());
    }

    // Apply CLI overrides before any API calls
    if let Some(model) = cli.model {
        cfg.model = model;
    }
    if let Some(api) = cli.api {
        cfg.api_url = api;
    }

    // --list
    if cli.list_models {
        let models = api::list_models(&cfg)?;
        if models.is_empty() {
            eprintln!("{}", "No models returned by API.".yellow());
        } else {
            for m in models {
                let state = m.state.as_deref().unwrap_or("unknown");
                let indicator = if state == "loaded" {
                    "●".green().to_string()
                } else {
                    "○".dimmed().to_string()
                };
                println!("{} {}", indicator, m.id);
            }
        }
        return Ok(());
    }

    // --load-model
    if let Some(ref model_id) = cli.load_model {
        eprint!("{}", format!("loading {model_id}...").dimmed());
        io::stderr().flush()?;
        api::load_model(&cfg, model_id)?;
        eprint!("\r\x1b[K");
        eprintln!("{} {}", "loaded:".green(), model_id);
        // If no prompt given, stop here
        if cli.prompt.is_empty() {
            return Ok(());
        }
    }

    if cli.prompt.is_empty() {
        eprintln!(
            "{}",
            "Usage: clat <what you want to do>\n       clat --help for more options".yellow()
        );
        std::process::exit(1);
    }

    let prompt = cli.prompt.join(" ");

    if cli.verbose {
        eprintln!("{} {}", "prompt:".dimmed(), prompt);
        eprintln!("{} {}", "api:   ".dimmed(), cfg.api_url);
        eprintln!("{} {}", "model: ".dimmed(), cfg.model);
        eprintln!("{} {}", "tools: ".dimmed(), cfg.use_tools);
    }

    // Generate script
    eprint!("{}", "thinking...".dimmed());
    io::stderr().flush()?;
    let script = api::generate_script(&cfg, &prompt)?;
    eprint!("\r\x1b[K");

    if script.trim().is_empty() {
        eprintln!("{}", "No script generated.".red());
        std::process::exit(1);
    }

    // Display
    let divider = "─".repeat(50).dimmed().to_string();
    println!("{}", divider);
    println!("{}", script.cyan());
    println!("{}", divider);

    // Warn if sudo is involved — OS will handle the password prompt, not us
    if executor::contains_sudo(&script) {
        eprintln!("{}", "note: script contains sudo — the OS will handle the password prompt".yellow());
    }

    if cli.dry_run {
        return Ok(());
    }

    let run = if cli.yes || cfg.auto_run || matches_auto_run_patterns(&script, &cfg.auto_run_patterns) {
        true
    } else {
        print!("Run? [y/N] ");
        io::stdout().flush()?;
        let mut line = String::new();
        io::stdin().read_line(&mut line)?;
        matches!(line.trim().to_lowercase().as_str(), "y" | "yes")
    };

    if run {
        let code = executor::run(&script)?;
        if code != 0 {
            std::process::exit(code);
        }
    } else {
        eprintln!("{}", "Aborted.".yellow());
    }

    Ok(())
}

/// Returns true if any non-comment line in the script starts with a command
/// name listed in `patterns`. When true, the confirmation prompt is skipped.
///
/// Example config:
///   auto_run_patterns = ["ls", "echo", "cat", "git", "brew"]
fn matches_auto_run_patterns(script: &str, patterns: &[String]) -> bool {
    if patterns.is_empty() {
        return false;
    }
    script
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .any(|line| {
            let cmd = line.split_whitespace().next().unwrap_or("");
            patterns.iter().any(|p| p == cmd)
        })
}
