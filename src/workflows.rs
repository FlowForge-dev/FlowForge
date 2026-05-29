use crate::config::{ForgeConfig, workflows_dir};
use crate::patterns::PatternEngine;
use crate::providers::ProviderRegistry;
use crate::terminal::{heading, info, success};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Workflow {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    pub steps: Vec<String>,
}

pub fn create_workflow(name: &str) -> Result<()> {
    validate_name(name)?;
    let path = workflow_path(name)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    if path.exists() {
        bail!("workflow `{name}` already exists")
    }
    let workflow = Workflow {
        name: name.to_string(),
        description: Some("Describe this workflow.".to_string()),
        steps: vec!["summarize".to_string(), "review".to_string()],
    };
    fs::write(path, serde_yaml::to_string(&workflow)?)?;
    Ok(())
}

pub async fn run_workflow(
    name: &str,
    input: &str,
    patterns: &PatternEngine,
    config: &ForgeConfig,
    providers: &ProviderRegistry,
) -> Result<String> {
    validate_name(name)?;
    let workflow = load_workflow(name)?;
    let mut current = input.to_string();
    for step in workflow.steps {
        current = patterns.run(&step, &current, config, providers).await?;
    }
    Ok(current)
}

pub fn print_workflows() -> Result<()> {
    heading("Workflows");
    for workflow in list_workflows()? {
        println!(
            "{:<24} {}",
            workflow.name,
            workflow
                .description
                .unwrap_or_else(|| "No description".to_string())
        );
    }
    Ok(())
}

pub fn edit_workflow(name: &str) -> Result<()> {
    validate_name(name)?;
    let path = workflow_path(name)?;
    if !path.exists() {
        create_workflow(name)?;
    }
    let editor = std::env::var("EDITOR").unwrap_or_else(|_| {
        if cfg!(windows) {
            "notepad".to_string()
        } else {
            "vi".to_string()
        }
    });
    Command::new(editor).arg(path).status()?;
    Ok(())
}

pub fn print_marketplace() -> Result<()> {
    heading("Workflow Marketplace");
    for workflow in list_marketplace_workflows()? {
        println!(
            "{:<24} {}",
            workflow.name,
            workflow
                .description
                .unwrap_or_else(|| "No description".to_string())
        );
    }
    Ok(())
}

pub fn install_marketplace_workflow(name: &str) -> Result<()> {
    validate_name(name)?;
    let source = PathBuf::from("examples")
        .join("workflows")
        .join(format!("{name}.yaml"));
    if !source.exists() {
        bail!("workflow `{name}` was not found in the local marketplace")
    }
    let target = workflow_path(name)?;
    if let Some(parent) = target.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::copy(source, &target)?;
    success(&format!("Installed workflow {name}"));
    Ok(())
}

pub fn publish_workflow(name: &str) -> Result<()> {
    validate_name(name)?;
    let path = workflow_path(name)?;
    if !path.exists() {
        bail!("workflow `{name}` does not exist in your workflow directory")
    }
    info("Workflow publish is prepared for registry integration; local validation passed.");
    Ok(())
}

fn load_workflow(name: &str) -> Result<Workflow> {
    validate_name(name)?;
    let user_path = workflow_path(name)?;
    if user_path.exists() {
        return read_workflow(&user_path);
    }

    let example_path = PathBuf::from("examples")
        .join("workflows")
        .join(format!("{name}.yaml"));
    if example_path.exists() {
        return read_workflow(&example_path);
    }

    bail!("unknown workflow `{name}`")
}

fn list_workflows() -> Result<Vec<Workflow>> {
    let mut workflows = Vec::new();
    for dir in [
        PathBuf::from("examples").join("workflows"),
        workflows_dir()?,
    ] {
        if !dir.exists() {
            continue;
        }
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            if entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml") {
                workflows.push(read_workflow(&entry.path())?);
            }
        }
    }
    workflows.sort_by(|a, b| a.name.cmp(&b.name));
    workflows.dedup_by(|a, b| a.name == b.name);
    Ok(workflows)
}

fn list_marketplace_workflows() -> Result<Vec<Workflow>> {
    let dir = PathBuf::from("examples").join("workflows");
    let mut workflows = Vec::new();
    if !dir.exists() {
        return Ok(workflows);
    }
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        if entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml") {
            workflows.push(read_workflow(&entry.path())?);
        }
    }
    workflows.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(workflows)
}

fn read_workflow(path: &Path) -> Result<Workflow> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read workflow {}", path.display()))?;
    serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse workflow {}", path.display()))
}

fn workflow_path(name: &str) -> Result<PathBuf> {
    Ok(workflows_dir()?.join(format!("{name}.yaml")))
}

fn validate_name(name: &str) -> Result<()> {
    let valid = !name.is_empty()
        && name.len() <= 64
        && name
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || ch == '-' || ch == '_');
    if valid {
        Ok(())
    } else {
        bail!("invalid workflow name `{name}`; use letters, numbers, hyphen, or underscore")
    }
}
