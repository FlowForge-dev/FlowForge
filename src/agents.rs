use crate::cli::AgentCommand;
use crate::config::ForgeConfig;
use crate::patterns::PatternEngine;
use crate::project;
use crate::providers::ProviderRegistry;
use crate::terminal::{heading, info};
use anyhow::Result;
use async_trait::async_trait;

#[derive(Debug, Clone)]
pub struct AgentContext {
    pub goal: String,
    pub project_context: String,
    pub prior_outputs: Vec<AgentOutput>,
}

#[derive(Debug, Clone)]
pub struct AgentOutput {
    pub agent: &'static str,
    pub content: String,
}

#[async_trait]
pub trait Agent: Send + Sync {
    fn name(&self) -> &'static str;
    fn pattern(&self) -> &'static str;

    async fn run(
        &self,
        ctx: AgentContext,
        patterns: &PatternEngine,
        config: &ForgeConfig,
        providers: &ProviderRegistry,
    ) -> Result<AgentOutput> {
        let prior_outputs = ctx
            .prior_outputs
            .iter()
            .map(|output| format!("## {}\n{}", output.agent, output.content))
            .collect::<Vec<_>>()
            .join("\n\n");
        let prompt = format!(
            "Goal: {}\n\nProject Index:\n{}\n\nPrior Agent Outputs:\n{}",
            ctx.goal, ctx.project_context, prior_outputs
        );
        let content = patterns
            .run(self.pattern(), &prompt, config, providers)
            .await?;
        Ok(AgentOutput {
            agent: self.name(),
            content,
        })
    }
}

struct BuiltinAgent {
    name: &'static str,
    pattern: &'static str,
}

#[async_trait]
impl Agent for BuiltinAgent {
    fn name(&self) -> &'static str {
        self.name
    }

    fn pattern(&self) -> &'static str {
        self.pattern
    }
}

pub async fn handle_agent_command(
    command: AgentCommand,
    patterns: &PatternEngine,
    config: &ForgeConfig,
    providers: &ProviderRegistry,
) -> Result<()> {
    let (title, goal, pipeline) = match command {
        AgentCommand::BuildApp(args) => (
            "Build App",
            args.goal
                .unwrap_or_else(|| "Plan and build the requested app".to_string()),
            vec![planner(), coder(), tester(), reviewer(), documenter()],
        ),
        AgentCommand::ReviewCode(args) => (
            "Review Code",
            args.goal
                .unwrap_or_else(|| "Review the current project".to_string()),
            vec![researcher(), reviewer()],
        ),
        AgentCommand::DebugProject(args) => (
            "Debug Project",
            args.goal
                .unwrap_or_else(|| "Debug the current project".to_string()),
            vec![planner(), researcher(), coder(), tester(), reviewer()],
        ),
    };

    heading(&format!("Agent Pipeline: {title}"));
    let project_context = project::index_context(".")?;
    let mut outputs = Vec::new();
    for agent in pipeline {
        info(&format!("{} running", agent.name()));
        let output = agent
            .run(
                AgentContext {
                    goal: goal.clone(),
                    project_context: project_context.clone(),
                    prior_outputs: outputs.clone(),
                },
                patterns,
                config,
                providers,
            )
            .await?;
        println!("\n## {}\n{}\n", output.agent, output.content);
        outputs.push(output);
    }
    Ok(())
}

fn planner() -> BuiltinAgent {
    BuiltinAgent {
        name: "Planner",
        pattern: "analyze",
    }
}

fn researcher() -> BuiltinAgent {
    BuiltinAgent {
        name: "Researcher",
        pattern: "architecture",
    }
}

fn coder() -> BuiltinAgent {
    BuiltinAgent {
        name: "Coder",
        pattern: "fix",
    }
}

fn tester() -> BuiltinAgent {
    BuiltinAgent {
        name: "Tester",
        pattern: "debug",
    }
}

fn reviewer() -> BuiltinAgent {
    BuiltinAgent {
        name: "Reviewer",
        pattern: "review",
    }
}

fn documenter() -> BuiltinAgent {
    BuiltinAgent {
        name: "Documenter",
        pattern: "explain",
    }
}
