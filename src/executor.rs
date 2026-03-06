use anyhow::Result;
use std::process::Command;

pub fn run(script: &str) -> Result<i32> {
    let status = Command::new("bash").arg("-c").arg(script).status()?;
    Ok(status.code().unwrap_or(1))
}

/// Returns true if the script contains a `sudo` invocation.
/// The OS will handle the password prompt — we just want to warn the user.
pub fn contains_sudo(script: &str) -> bool {
    script.lines().any(|line| {
        let t = line.trim();
        if t.starts_with('#') {
            return false;
        }
        t == "sudo"
            || t.starts_with("sudo ")
            || t.contains(" sudo ")
            || t.contains("\tsudo ")
    })
}
