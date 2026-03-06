use anyhow::Result;
use std::process::Command;

pub fn run(script: &str) -> Result<i32> {
    let status = Command::new("bash").arg("-c").arg(script).status()?;
    Ok(status.code().unwrap_or(1))
}
