use crate::terminal::heading;
use anyhow::{Context, Result, bail};
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};
use tokio::process::Command;

pub fn search(query: &str, path: Option<&Path>) -> Result<()> {
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

pub async fn shell(command: Vec<String>) -> Result<()> {
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

pub fn write_file(path: &PathBuf, text: &str) -> Result<()> {
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
