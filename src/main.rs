use std::io::{self, Write};

use anyhow::Result;
use clap::Parser;
use colored::Colorize;

mod api;
mod config;
mod executor;

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

    /// Verbose: print config info before running
    #[arg(short, long)]
    verbose: bool,

    /// Show config path and current settings
    #[arg(long)]
    config: bool,

    /// Write default config file (won't overwrite existing)
    #[arg(long)]
    init: bool,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut cfg = config::Config::load()?;

    // --init: create config file if it doesn't exist
    if cli.init {
        let path = config::Config::path();
        if path.exists() {
            println!("Config already exists at: {}", path.display());
        } else {
            cfg.save()?;
            println!("Created config at: {}", path.display());
            println!("Edit it to set your API URL, model, and optionally auto_run = true.");
        }
        return Ok(());
    }

    // --config: show current settings
    if cli.config {
        let path = config::Config::path();
        println!("{} {}", "Config:".dimmed(), path.display());
        println!("{}", toml::to_string_pretty(&cfg)?);
        return Ok(());
    }

    if cli.prompt.is_empty() {
        eprintln!(
            "{}",
            "Usage: clat <what you want to do>\n       clat --help for more options".yellow()
        );
        std::process::exit(1);
    }

    // Apply CLI overrides
    if let Some(model) = cli.model {
        cfg.model = model;
    }
    if let Some(api) = cli.api {
        cfg.api_url = api;
    }

    let prompt = cli.prompt.join(" ");

    if cli.verbose {
        eprintln!("{} {}", "prompt:".dimmed(), prompt);
        eprintln!("{} {}", "api:   ".dimmed(), cfg.api_url);
        eprintln!("{} {}", "model: ".dimmed(), cfg.model);
    }

    // Generate script
    eprint!("{}", "thinking...".dimmed());
    io::stderr().flush()?;
    let script = api::generate_script(&cfg, &prompt)?;
    eprint!("\r\x1b[K"); // clear the "thinking..." line

    if script.trim().is_empty() {
        eprintln!("{}", "No script generated.".red());
        std::process::exit(1);
    }

    // Display
    let divider = "─".repeat(50).dimmed().to_string();
    println!("{}", divider);
    println!("{}", script.cyan());
    println!("{}", divider);

    if cli.dry_run {
        return Ok(());
    }

    let run = if cli.yes || cfg.auto_run {
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
