use anyhow::{Context, Result};
use clap::{Args, Parser, Subcommand};
use std::io::{self, IsTerminal, Read};
use std::path::PathBuf;

#[derive(Debug, Parser)]
#[command(
    name = "forge",
    version,
    about = "Terminal-first AI automation for developers"
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    Init,
    Provider {
        #[command(subcommand)]
        command: ProviderCommand,
    },
    Pattern {
        #[command(subcommand)]
        command: PatternCommand,
    },
    Run {
        pattern: String,
        #[arg(short, long)]
        text: Option<String>,
    },
    Chain {
        patterns: Vec<String>,
        #[arg(short, long)]
        text: Option<String>,
    },
    Workflow {
        #[command(subcommand)]
        command: WorkflowCommand,
    },
    Agent {
        #[command(subcommand)]
        command: AgentCommand,
    },
    Analyze,
    ExplainProject,
    FindDeadCode,
    Architecture,
    GenerateDocs,
    Remember {
        note: Vec<String>,
    },
    Recall,
    Memory {
        #[command(subcommand)]
        command: MemoryCommand,
    },
    Plugin {
        #[command(subcommand)]
        command: PluginCommand,
    },
    Tool {
        #[command(subcommand)]
        command: ToolCommand,
    },
    Chat,
}

#[derive(Debug, Subcommand)]
pub enum ProviderCommand {
    Set {
        provider: String,
        #[arg(short, long)]
        model: Option<String>,
    },
    List,
    Test {
        provider: Option<String>,
    },
    Info {
        provider: String,
    },
    Show,
}

#[derive(Debug, Subcommand)]
pub enum PatternCommand {
    Create { name: String },
    Edit { name: String },
    List,
    Install { path: PathBuf },
    Publish { name: String },
    Remove { name: String },
}

#[derive(Debug, Subcommand)]
pub enum WorkflowCommand {
    Create {
        name: String,
    },
    Run {
        name: String,
        #[arg(short, long)]
        text: Option<String>,
    },
    List,
    Edit {
        name: String,
    },
    Marketplace,
    Install {
        name: String,
    },
    Publish {
        name: String,
    },
}

#[derive(Debug, Subcommand)]
pub enum AgentCommand {
    BuildApp(AgentGoal),
    ReviewCode(AgentGoal),
    DebugProject(AgentGoal),
}

#[derive(Debug, Args)]
pub struct AgentGoal {
    #[arg(short, long)]
    pub goal: Option<String>,
}

#[derive(Debug, Subcommand)]
pub enum MemoryCommand {
    Search { query: Vec<String> },
    Clear,
}

#[derive(Debug, Subcommand)]
pub enum PluginCommand {
    Install { source: String },
    Remove { name: String },
    Update { name: String },
    Scaffold { name: String },
    Validate { path: Option<PathBuf> },
    List,
}

#[derive(Debug, Subcommand)]
pub enum ToolCommand {
    Search {
        query: String,
        #[arg(short, long)]
        path: Option<PathBuf>,
    },
    Git {
        args: Vec<String>,
    },
    Http {
        url: String,
    },
    Shell {
        command: Vec<String>,
    },
    Read {
        path: PathBuf,
    },
    Write {
        path: PathBuf,
        text: String,
    },
    Scan {
        path: Option<PathBuf>,
    },
}

pub fn read_input(text: Option<String>) -> Result<String> {
    if let Some(text) = text {
        return Ok(text);
    }

    if !io::stdin().is_terminal() {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .context("failed to read stdin")?;
        return Ok(buffer);
    }

    Ok(String::new())
}
