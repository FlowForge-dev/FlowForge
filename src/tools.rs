use crate::config::ForgeConfig;
use crate::terminal::heading;
use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub fn search(query: &str, path: Option<&Path>, config: &ForgeConfig) -> Result<()> {
    let root = path.unwrap_or_else(|| Path::new("."));
    heading("Search");
    for result in WalkBuilder::new(root)
        .hidden(false)
        .filter_entry(|entry| !is_generated_dir(entry.path()))
        .build()
    {
        let entry = result?;
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        if is_too_large(path, config.tools.max_search_file_bytes)? {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for (idx, line) in content.lines().enumerate() {
            if line.contains(query) {
                println!("{}:{} {}", path.display(), idx + 1, line.trim());
            }
        }
    }
    Ok(())
}

pub async fn git(args: Vec<String>) -> Result<()> {
    let output = Command::new("git").args(args).output().await?;
    print_output(output)?;
    Ok(())
}

pub async fn http_get(url: &str) -> Result<()> {
    let text = reqwest::get(url).await?.text().await?;
    println!("{text}");
    Ok(())
}

pub async fn shell(command: Vec<String>, config: &ForgeConfig) -> Result<()> {
    if !config.tools.shell_enabled {
        bail!(
            "shell tool is disabled; set [tools].shell_enabled = true in ~/.forgeflow/config.toml to opt in"
        )
    }
    if command.is_empty() {
        bail!("shell command required")
    }
    let (program, args) = command.split_first().context("shell command required")?;
    let output = Command::new(program).args(args).output().await?;
    print_output(output)?;
    Ok(())
}

pub fn read_file(path: &PathBuf) -> Result<()> {
    println!("{}", std::fs::read_to_string(path)?);
    Ok(())
}

pub fn write_file(path: &PathBuf, text: &str, config: &ForgeConfig) -> Result<()> {
    ensure_write_allowed(path, config)?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    std::fs::write(path, text)?;
    Ok(())
}

fn print_output(output: std::process::Output) -> Result<()> {
    print!("{}", String::from_utf8_lossy(&output.stdout));
    eprint!("{}", String::from_utf8_lossy(&output.stderr));
    if !output.status.success() {
        bail!("command exited with {}", output.status)
    }
    Ok(())
}

fn is_generated_dir(path: &Path) -> bool {
    let Some(name) = path.file_name().and_then(|name| name.to_str()) else {
        return false;
    };
    matches!(
        name,
        ".git" | ".forgeflow" | ".tmp-forgeflow-home" | "node_modules" | "target"
    )
}

fn ensure_write_allowed(path: &Path, config: &ForgeConfig) -> Result<()> {
    if config.tools.allow_write_outside_workspace {
        return Ok(());
    }
    let cwd = std::env::current_dir()?.canonicalize()?;
    let target = if path.exists() {
        path.canonicalize()?
    } else {
        let parent = path.parent().unwrap_or_else(|| Path::new("."));
        parent
            .canonicalize()?
            .join(path.file_name().unwrap_or_default())
    };
    if target.starts_with(&cwd) {
        return Ok(());
    }
    bail!(
        "refusing to write outside the current workspace; set [tools].allow_write_outside_workspace = true to opt in"
    )
}

fn is_too_large(path: &Path, max_bytes: u64) -> Result<bool> {
    Ok(path.metadata()?.len() > max_bytes)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn shell_is_disabled_by_default() {
        let result = shell(
            vec!["echo".to_string(), "hello".to_string()],
            &ForgeConfig::default(),
        )
        .await;
        assert!(result.is_err());
    }
}
