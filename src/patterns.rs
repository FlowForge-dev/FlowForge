use crate::config::{ForgeConfig, patterns_dir};
use crate::providers::{CompletionRequest, ProviderRegistry};
use crate::terminal::{heading, info};
use anyhow::{Context, Result, bail};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub name: String,
    pub description: String,
    pub system_prompt: String,
    pub user_template: String,
}

#[derive(Debug, Default)]
pub struct PatternEngine;

impl PatternEngine {
    pub async fn run(
        &self,
        name: &str,
        input: &str,
        config: &ForgeConfig,
        providers: &ProviderRegistry,
    ) -> Result<String> {
        let pattern = self.load_pattern(name)?;
        let user_prompt = pattern.user_template.replace("{{input}}", input);
        providers
            .complete(
                CompletionRequest {
                    system_prompt: pattern.system_prompt,
                    user_prompt,
                },
                config,
            )
            .await
    }

    pub async fn run_stream(
        &self,
        name: &str,
        input: &str,
        config: &ForgeConfig,
        providers: &ProviderRegistry,
    ) -> Result<String> {
        let pattern = self.load_pattern(name)?;
        let user_prompt = pattern.user_template.replace("{{input}}", input);
        let mut on_delta = |delta: &str| {
            print!("{delta}");
            let _ = std::io::Write::flush(&mut std::io::stdout());
        };
        providers
            .complete_stream(
                CompletionRequest {
                    system_prompt: pattern.system_prompt,
                    user_prompt,
                },
                config,
                &mut on_delta,
            )
            .await
    }

    pub async fn chain_stream(
        &self,
        names: &[String],
        input: &str,
        config: &ForgeConfig,
        providers: &ProviderRegistry,
    ) -> Result<String> {
        if names.is_empty() {
            bail!("chain requires at least one pattern")
        }

        let mut current = input.to_string();
        let last = names.len() - 1;
        for (idx, name) in names.iter().enumerate() {
            info(&format!("running pattern {name}"));
            current = if idx == last {
                self.run_stream(name, &current, config, providers).await?
            } else {
                self.run(name, &current, config, providers).await?
            };
        }
        Ok(current)
    }

    pub fn print_patterns(&self) -> Result<()> {
        heading("Patterns");
        for pattern in self.list_patterns()? {
            println!("{:<16} {}", pattern.name, pattern.description);
        }
        Ok(())
    }

    pub fn list_patterns(&self) -> Result<Vec<Pattern>> {
        let mut patterns = builtin_patterns();
        let dir = patterns_dir()?;
        if dir.exists() {
            for entry in fs::read_dir(&dir)? {
                let entry = entry?;
                if entry.path().extension().and_then(|ext| ext.to_str()) == Some("yaml") {
                    let pattern = read_pattern(&entry.path())?;
                    patterns.retain(|existing| existing.name != pattern.name);
                    patterns.push(pattern);
                }
            }
        }
        patterns.sort_by(|a, b| a.name.cmp(&b.name));
        Ok(patterns)
    }

    pub fn create_user_pattern(&self, name: &str) -> Result<()> {
        validate_name(name)?;
        let path = user_pattern_path(name)?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        if path.exists() {
            bail!("pattern `{name}` already exists")
        }
        let pattern = Pattern {
            name: name.to_string(),
            description: "Describe what this pattern does.".to_string(),
            system_prompt: "You are a helpful developer assistant.".to_string(),
            user_template: "Process the following input:\n\n{{input}}".to_string(),
        };
        fs::write(&path, serde_yaml::to_string(&pattern)?)?;
        Ok(())
    }

    pub fn edit_user_pattern(&self, name: &str) -> Result<()> {
        let path = user_pattern_path(name)?;
        if !path.exists() {
            self.create_user_pattern(name)?;
        }
        open_editor(&path)
    }

    pub fn install_pattern(&self, path: &Path) -> Result<()> {
        let pattern = read_pattern(path)?;
        validate_name(&pattern.name)?;
        let target = user_pattern_path(&pattern.name)?;
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(path, target)?;
        Ok(())
    }

    pub fn remove_user_pattern(&self, name: &str) -> Result<()> {
        validate_name(name)?;
        let path = user_pattern_path(name)?;
        if path.exists() {
            fs::remove_file(path)?;
        }
        Ok(())
    }

    fn load_pattern(&self, name: &str) -> Result<Pattern> {
        validate_name(name)?;
        for pattern in self.list_patterns()? {
            if pattern.name == name {
                return Ok(pattern);
            }
        }
        bail!("unknown pattern `{name}`")
    }
}

fn read_pattern(path: &Path) -> Result<Pattern> {
    let raw = fs::read_to_string(path)
        .with_context(|| format!("failed to read pattern {}", path.display()))?;
    serde_yaml::from_str(&raw)
        .with_context(|| format!("failed to parse pattern {}", path.display()))
}

fn user_pattern_path(name: &str) -> Result<PathBuf> {
    Ok(patterns_dir()?.join(format!("{name}.yaml")))
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
        bail!("invalid pattern name `{name}`; use letters, numbers, hyphen, or underscore")
    }
}

fn open_editor(path: &Path) -> Result<()> {
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

fn builtin_patterns() -> Vec<Pattern> {
    vec![
        Pattern {
            name: "summarize".into(),
            description: "Condense technical input into the most useful points.".into(),
            system_prompt: "You summarize technical material for developers with crisp structure.".into(),
            user_template: "Summarize the following content for a developer:\n\n{{input}}".into(),
        },
        Pattern {
            name: "debug".into(),
            description: "Identify likely causes and fixes for errors or logs.".into(),
            system_prompt: "You are a pragmatic debugging assistant.".into(),
            user_template: "Analyze this debugging context and identify root causes and next commands:\n\n{{input}}".into(),
        },
        Pattern {
            name: "architecture".into(),
            description: "Explain software architecture and tradeoffs.".into(),
            system_prompt: "You explain architecture clearly for senior developers.".into(),
            user_template: "Produce an architecture summary, components, data flow, risks, and extension points:\n\n{{input}}".into(),
        },
        Pattern {
            name: "explain".into(),
            description: "Explain code, commands, or technical concepts.".into(),
            system_prompt: "You explain technical material with accuracy and brevity.".into(),
            user_template: "Explain this for a developer:\n\n{{input}}".into(),
        },
        Pattern {
            name: "refactor".into(),
            description: "Suggest targeted behavior-preserving refactors.".into(),
            system_prompt: "You are a careful refactoring assistant.".into(),
            user_template: "Review this code or design and suggest focused refactors:\n\n{{input}}".into(),
        },
        Pattern {
            name: "quiz".into(),
            description: "Convert material into study questions.".into(),
            system_prompt: "You create concise technical quizzes.".into(),
            user_template: "Create a short quiz from this material:\n\n{{input}}".into(),
        },
        Pattern {
            name: "flashcards".into(),
            description: "Convert material into flashcards.".into(),
            system_prompt: "You create useful developer flashcards.".into(),
            user_template: "Convert this material into front/back flashcards:\n\n{{input}}".into(),
        },
        Pattern {
            name: "analyze".into(),
            description: "Analyze project or problem context.".into(),
            system_prompt: "You analyze developer context and produce actionable findings.".into(),
            user_template: "Analyze this context:\n\n{{input}}".into(),
        },
        Pattern {
            name: "fix".into(),
            description: "Propose a precise fix plan.".into(),
            system_prompt: "You propose careful code fixes with verification steps.".into(),
            user_template: "Propose a fix for this context:\n\n{{input}}".into(),
        },
        Pattern {
            name: "review".into(),
            description: "Review a solution for risks and gaps.".into(),
            system_prompt: "You review code and plans for correctness, regressions, and missing tests.".into(),
            user_template: "Review this result:\n\n{{input}}".into(),
        },
    ]
}
