mod agents;
mod cli;
mod config;
mod memory;
mod patterns;
mod plugins;
mod project;
mod providers;
mod terminal;
mod tools;
mod workflows;

use anyhow::Result;
use clap::Parser;
use cli::{
    Cli, Commands, MemoryCommand, PatternCommand, ProviderCommand, ToolCommand, WorkflowCommand,
};
use config::ForgeConfig;
use memory::MemoryStore;
use patterns::PatternEngine;
use providers::ProviderRegistry;
use terminal::{heading, info, success};

#[tokio::main]
async fn main() -> Result<()> {
    let cli = Cli::parse();
    let mut config = ForgeConfig::load_or_init()?;
    let memory = MemoryStore::open_default()?;
    let providers = ProviderRegistry::new();
    let patterns = PatternEngine;

    match cli.command {
        Commands::Init => init_home()?,
        Commands::Provider { command } => handle_provider(command, &mut config, &providers).await?,
        Commands::Pattern { command } => handle_pattern(command, &patterns)?,
        Commands::Run { pattern, text } => {
            let input = cli::read_input(text)?;
            patterns
                .run_stream(&pattern, &input, &config, &providers)
                .await?;
            println!();
        }
        Commands::Chain {
            patterns: names,
            text,
        } => {
            let input = cli::read_input(text)?;
            patterns
                .chain_stream(&names, &input, &config, &providers)
                .await?;
            println!();
        }
        Commands::Workflow { command } => {
            handle_workflow(command, &patterns, &config, &providers).await?
        }
        Commands::Agent { command } => {
            agents::handle_agent_command(command, &patterns, &config, &providers).await?
        }
        Commands::Analyze => project::print_project_scan(".", &config)?,
        Commands::ExplainProject => {
            project::explain_project(&patterns, &config, &providers).await?
        }
        Commands::FindDeadCode => project::find_dead_code(".")?,
        Commands::Architecture => project::architecture(&patterns, &config, &providers).await?,
        Commands::GenerateDocs => project::generate_docs(&patterns, &config, &providers).await?,
        Commands::Remember { note } => {
            let content = note.join(" ");
            memory.remember("note", &content, None)?;
            success("Saved memory");
        }
        Commands::Recall => memory.print_recent(10)?,
        Commands::Memory { command } => handle_memory(command, &memory)?,
        Commands::Plugin { command } => plugins::handle_plugin_command(command)?,
        Commands::Tool { command } => handle_tool(command, &config).await?,
        Commands::Chat => chat(&config, &providers).await?,
    }

    Ok(())
}

fn init_home() -> Result<()> {
    config::init_home_layout()?;
    success("ForgeFlow is ready");
    println!("home      {}", config::forge_home()?.display());
    println!("config    {}", config::config_path()?.display());
    println!("patterns  {}", config::patterns_dir()?.display());
    println!("workflows {}", config::workflows_dir()?.display());
    println!("plugins   {}", config::plugins_dir()?.display());
    Ok(())
}

async fn handle_provider(
    command: ProviderCommand,
    config: &mut ForgeConfig,
    providers: &ProviderRegistry,
) -> Result<()> {
    match command {
        ProviderCommand::Set { provider, model } => {
            providers.ensure_supported(&provider)?;
            config.provider = provider.clone();
            if let Some(model) = model {
                config.model = model;
            }
            config.save()?;
            success(&format!("Provider set to {provider}"));
        }
        ProviderCommand::List => {
            providers.print_supported(config);
            providers.print_health(None, config).await?;
        }
        ProviderCommand::Test { provider } => {
            providers.test(provider.as_deref(), config).await?;
        }
        ProviderCommand::Info { provider } => providers.print_info(&provider, config)?,
        ProviderCommand::Show => {
            heading("Active Provider");
            println!("provider = {}", config.provider);
            println!("model    = {}", config.model);
        }
    }
    Ok(())
}

async fn chat(config: &ForgeConfig, providers: &ProviderRegistry) -> Result<()> {
    use std::io::Write;

    heading("ForgeFlow Chat");
    info("Type /exit to leave.");
    let mut transcript = String::new();
    loop {
        print!("\nyou> ");
        std::io::stdout().flush()?;
        let mut line = String::new();
        if std::io::stdin().read_line(&mut line)? == 0 {
            break;
        }
        let line = line.trim();
        if line.eq_ignore_ascii_case("/exit") || line.eq_ignore_ascii_case("exit") {
            break;
        }
        if line.is_empty() {
            continue;
        }

        transcript.push_str("\nUser: ");
        transcript.push_str(line);
        transcript.push_str("\nAssistant: ");
        print!("forge> ");
        std::io::stdout().flush()?;
        let mut on_delta = |delta: &str| {
            print!("{delta}");
            let _ = std::io::stdout().flush();
        };
        let output = providers
            .complete_stream(
                providers::CompletionRequest {
                    system_prompt:
                        "You are ForgeFlow, a concise terminal-first AI assistant for developers."
                            .to_string(),
                    user_prompt: transcript.clone(),
                },
                config,
                &mut on_delta,
            )
            .await?;
        println!();
        transcript.push_str(&output);
    }
    Ok(())
}

fn handle_pattern(command: PatternCommand, engine: &PatternEngine) -> Result<()> {
    match command {
        PatternCommand::Create { name } => {
            engine.create_user_pattern(&name)?;
            success(&format!("Created pattern {name}"));
        }
        PatternCommand::Edit { name } => engine.edit_user_pattern(&name)?,
        PatternCommand::List => engine.print_patterns()?,
        PatternCommand::Install { path } => {
            engine.install_pattern(&path)?;
            success("Installed pattern");
        }
        PatternCommand::Publish { name } => {
            info(&format!(
                "Pattern publishing is scaffolded for {name}. Registry support lands in plugin phase."
            ));
        }
        PatternCommand::Remove { name } => {
            engine.remove_user_pattern(&name)?;
            success(&format!("Removed pattern {name}"));
        }
    }
    Ok(())
}

async fn handle_workflow(
    command: WorkflowCommand,
    patterns: &PatternEngine,
    config: &ForgeConfig,
    providers: &ProviderRegistry,
) -> Result<()> {
    match command {
        WorkflowCommand::Create { name } => {
            workflows::create_workflow(&name)?;
            success(&format!("Created workflow {name}"));
        }
        WorkflowCommand::Run { name, text } => {
            let input = cli::read_input(text)?;
            let output =
                workflows::run_workflow(&name, &input, patterns, config, providers).await?;
            println!("{output}");
        }
        WorkflowCommand::List => workflows::print_workflows()?,
        WorkflowCommand::Edit { name } => workflows::edit_workflow(&name)?,
        WorkflowCommand::Marketplace => workflows::print_marketplace()?,
        WorkflowCommand::Install { name } => workflows::install_marketplace_workflow(&name)?,
        WorkflowCommand::Publish { name } => workflows::publish_workflow(&name)?,
    }
    Ok(())
}

fn handle_memory(command: MemoryCommand, memory: &MemoryStore) -> Result<()> {
    match command {
        MemoryCommand::Search { query } => memory.search(&query.join(" "))?,
        MemoryCommand::Clear => {
            memory.clear()?;
            success("Cleared memory");
        }
    }
    Ok(())
}

async fn handle_tool(command: ToolCommand, config: &ForgeConfig) -> Result<()> {
    match command {
        ToolCommand::Search { query, path } => tools::search(&query, path.as_deref(), config)?,
        ToolCommand::Git { args } => tools::git(args).await?,
        ToolCommand::Http { url } => tools::http_get(&url).await?,
        ToolCommand::Shell { command } => tools::shell(command, config).await?,
        ToolCommand::Read { path } => tools::read_file(&path)?,
        ToolCommand::Write { path, text } => tools::write_file(&path, &text, config)?,
        ToolCommand::Scan { path } => {
            let path = path.unwrap_or_else(|| ".".into());
            project::print_project_scan(path, config)?
        }
    }
    Ok(())
}
