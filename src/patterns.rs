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
        Pattern {
            name: "distill".into(),
            description: "Extract the highest-value ideas, actions, and reusable knowledge.".into(),
            system_prompt: "You convert messy source material into a clean knowledge brief for a busy developer.".into(),
            user_template: "Distill this material into: one-line summary, core ideas, useful details, action items, questions to revisit, and a memorable takeaway. Keep it practical and avoid filler.\n\n{{input}}".into(),
        },
        Pattern {
            name: "key-points".into(),
            description: "Pull out the essential points without commentary.".into(),
            system_prompt: "You extract only the essential points from source material.".into(),
            user_template: "Extract the essential points. Group related points. Remove repetition. Do not add ideas that are not supported by the input.\n\n{{input}}".into(),
        },
        Pattern {
            name: "insight-map".into(),
            description: "Map ideas, implications, tradeoffs, and follow-up questions.".into(),
            system_prompt: "You build clear idea maps from dense technical and business material.".into(),
            user_template: "Create an insight map with: central thesis, supporting ideas, hidden assumptions, implications, tradeoffs, open questions, and best next move.\n\n{{input}}".into(),
        },
        Pattern {
            name: "action-items".into(),
            description: "Turn input into concrete tasks with owners and priority hints.".into(),
            system_prompt: "You turn unstructured notes into concrete execution tasks.".into(),
            user_template: "Extract action items. For each item include: task, why it matters, suggested owner if obvious, priority, and dependency. If no action exists, say so plainly.\n\n{{input}}".into(),
        },
        Pattern {
            name: "decision-brief".into(),
            description: "Prepare a concise decision memo from messy context.".into(),
            system_prompt: "You write short decision briefs for technical teams.".into(),
            user_template: "Create a decision brief with: decision needed, context, options, recommendation, risks, unknowns, and what would change the recommendation.\n\n{{input}}".into(),
        },
        Pattern {
            name: "study-notes".into(),
            description: "Create beginner-friendly notes from technical material.".into(),
            system_prompt: "You turn technical material into beginner-friendly study notes.".into(),
            user_template: "Create study notes with: plain-English explanation, vocabulary, examples, common mistakes, quick quiz, and what to learn next.\n\n{{input}}".into(),
        },
        Pattern {
            name: "concept-map".into(),
            description: "Identify concepts and how they relate.".into(),
            system_prompt: "You identify concepts and relationships in technical material.".into(),
            user_template: "Build a concept map. List concepts, definitions, relationships, prerequisites, examples, and confusing lookalikes.\n\n{{input}}".into(),
        },
        Pattern {
            name: "claim-check".into(),
            description: "Separate claims, evidence, assumptions, and weak spots.".into(),
            system_prompt: "You evaluate claims carefully without pretending to know facts not in evidence.".into(),
            user_template: "Extract claims from this material. For each claim include: evidence in the input, confidence, assumptions, missing proof, and verification step.\n\n{{input}}".into(),
        },
        Pattern {
            name: "counterpoints".into(),
            description: "Generate thoughtful objections and alternative interpretations.".into(),
            system_prompt: "You stress-test ideas with fair, useful counterarguments.".into(),
            user_template: "Find the strongest counterpoints, alternative explanations, edge cases, and failure modes. Keep the tone constructive.\n\n{{input}}".into(),
        },
        Pattern {
            name: "quote-bank".into(),
            description: "Extract short useful quotes and explain why they matter.".into(),
            system_prompt: "You extract compact, useful quotes from provided text without inventing wording.".into(),
            user_template: "Extract the most useful short quotes from the input. For each quote include why it matters and where it could be used. Do not fabricate quotes.\n\n{{input}}".into(),
        },
        Pattern {
            name: "question-set".into(),
            description: "Create sharp questions for interviews, reviews, or learning.".into(),
            system_prompt: "You create questions that reveal understanding and gaps.".into(),
            user_template: "Create questions from this material. Group them into: beginner, practical, deep, skeptical, and follow-up questions.\n\n{{input}}".into(),
        },
        Pattern {
            name: "checklist".into(),
            description: "Convert input into a practical checklist.".into(),
            system_prompt: "You turn guidance into checklists people can actually follow.".into(),
            user_template: "Create a checklist from this material. Use short checkable items, group them by phase, and include common failure points.\n\n{{input}}".into(),
        },
        Pattern {
            name: "meeting-brief".into(),
            description: "Summarize meetings into decisions, actions, and follow-ups.".into(),
            system_prompt: "You turn meeting notes and transcripts into clear team briefs.".into(),
            user_template: "Create a meeting brief with: summary, decisions, action items, blockers, open questions, and follow-up message.\n\n{{input}}".into(),
        },
        Pattern {
            name: "research-brief".into(),
            description: "Turn notes into a research brief with gaps and next sources.".into(),
            system_prompt: "You organize research notes into a useful brief for follow-up work.".into(),
            user_template: "Create a research brief with: topic, findings, evidence, contradictions, missing information, next searches, and recommended direction.\n\n{{input}}".into(),
        },
        Pattern {
            name: "newbie-explain".into(),
            description: "Explain material for an absolute beginner.".into(),
            system_prompt: "You explain hard material to beginners without talking down to them.".into(),
            user_template: "Explain this for a beginner. Use simple words, one small example, a memory trick, and a tiny practice task.\n\n{{input}}".into(),
        },
        Pattern {
            name: "risks".into(),
            description: "Find risks, failure modes, and mitigations.".into(),
            system_prompt: "You find practical risks and mitigations in plans, code, and strategy.".into(),
            user_template: "Identify risks. For each risk include: trigger, impact, likelihood, warning sign, and mitigation.\n\n{{input}}".into(),
        },
        Pattern {
            name: "next-steps".into(),
            description: "Recommend the next few useful moves.".into(),
            system_prompt: "You convert ambiguous context into a short next-step plan.".into(),
            user_template: "Recommend the next steps. Keep it short. Include immediate action, later action, what to ignore, and a quick success check.\n\n{{input}}".into(),
        },
        Pattern {
            name: "content-ideas".into(),
            description: "Turn source material into ethical content angles and hooks.".into(),
            system_prompt: "You help create honest developer content without hype or plagiarism.".into(),
            user_template: "Create original content ideas from this material: hooks, titles, demos, short posts, long-form outline, and what not to exaggerate.\n\n{{input}}".into(),
        },
    ]
}
