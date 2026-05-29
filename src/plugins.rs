use crate::cli::PluginCommand;
use crate::config::plugins_dir;
use crate::terminal::{heading, info, success};
use anyhow::{Context, Result, bail};
use serde::Deserialize;
use std::fs;
use std::path::Path;

#[derive(Debug, Deserialize)]
struct PluginManifest {
    name: String,
    version: String,
    #[serde(default)]
    capabilities: PluginCapabilities,
}

#[derive(Debug, Default, Deserialize)]
struct PluginCapabilities {
    #[serde(default)]
    patterns: bool,
    #[serde(default)]
    agents: bool,
    #[serde(default)]
    providers: bool,
    #[serde(default)]
    tools: bool,
    #[serde(default)]
    workflow_steps: bool,
    #[serde(default)]
    commands: bool,
}

pub fn handle_plugin_command(command: PluginCommand) -> Result<()> {
    match command {
        PluginCommand::Install { source } => {
            info(&format!("Plugin install scaffolded for source: {source}"));
            info("MVP discovers local plugin manifests; registry download arrives in phase 5.");
        }
        PluginCommand::Remove { name } => {
            let path = plugins_dir()?.join(&name);
            if path.exists() {
                fs::remove_dir_all(path)?;
            }
            success(&format!("Removed plugin {name}"));
        }
        PluginCommand::Update { name } => {
            info(&format!("Plugin update scaffolded for {name}"));
        }
        PluginCommand::Scaffold { name } => scaffold_plugin(&name)?,
        PluginCommand::Validate { path } => {
            validate_plugin(path.as_deref().unwrap_or(Path::new(".")))?;
            success("Plugin manifest is valid");
        }
        PluginCommand::List => print_plugins()?,
    }
    Ok(())
}

fn scaffold_plugin(name: &str) -> Result<()> {
    let plugin_dir = plugins_dir()?.join(name);
    fs::create_dir_all(plugin_dir.join("patterns"))?;
    fs::create_dir_all(plugin_dir.join("workflows"))?;
    fs::create_dir_all(plugin_dir.join("tools"))?;
    fs::create_dir_all(plugin_dir.join("bin"))?;
    let manifest = format!(
        r#"name = "{name}"
version = "0.1.0"
description = "ForgeFlow plugin"

[capabilities]
patterns = true
agents = false
providers = false
tools = true
workflow_steps = true
commands = false
"#
    );
    fs::write(plugin_dir.join("plugin.toml"), manifest)?;
    fs::write(
        plugin_dir.join("README.md"),
        format!("# {name}\n\nA ForgeFlow plugin.\n"),
    )?;
    success(&format!("Scaffolded plugin at {}", plugin_dir.display()));
    Ok(())
}

fn validate_plugin(path: &Path) -> Result<()> {
    let manifest_path = path.join("plugin.toml");
    if !manifest_path.exists() {
        bail!("missing plugin.toml in {}", path.display());
    }
    let raw = fs::read_to_string(&manifest_path)
        .with_context(|| format!("failed to read {}", manifest_path.display()))?;
    let manifest: PluginManifest = toml::from_str(&raw)
        .with_context(|| format!("failed to parse {}", manifest_path.display()))?;
    if manifest.name.trim().is_empty() {
        bail!("plugin name cannot be empty");
    }
    if manifest.version.trim().is_empty() {
        bail!("plugin version cannot be empty");
    }
    let has_capability = manifest.capabilities.patterns
        || manifest.capabilities.agents
        || manifest.capabilities.providers
        || manifest.capabilities.tools
        || manifest.capabilities.workflow_steps
        || manifest.capabilities.commands;
    if !has_capability {
        bail!("plugin must declare at least one capability");
    }
    Ok(())
}

fn print_plugins() -> Result<()> {
    heading("Plugins");
    let dir = plugins_dir()?;
    if !dir.exists() {
        info("No plugins installed");
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let manifest_path = entry.path().join("plugin.toml");
        if manifest_path.exists() {
            let raw = fs::read_to_string(&manifest_path)?;
            let manifest: PluginManifest = toml::from_str(&raw)?;
            println!("{:<20} {}", manifest.name, manifest.version);
        }
    }
    Ok(())
}
