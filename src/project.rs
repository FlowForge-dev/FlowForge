use crate::config::ForgeConfig;
use crate::patterns::PatternEngine;
use crate::providers::ProviderRegistry;
use crate::terminal::heading;
use anyhow::{Context, Result};
use ignore::WalkBuilder;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectIndex {
    pub root: String,
    pub files: Vec<IndexedFile>,
    pub extensions: BTreeMap<String, usize>,
    pub manifests: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IndexedFile {
    pub path: String,
    pub extension: Option<String>,
    pub bytes: u64,
    pub lines: Option<usize>,
    pub sha256: String,
    pub symbols: Vec<String>,
}

pub fn build_index(path: impl AsRef<Path>) -> Result<ProjectIndex> {
    let root = path.as_ref();
    let mut files = Vec::new();
    let mut extensions = BTreeMap::new();
    let mut manifests = Vec::new();

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
        let bytes =
            std::fs::read(path).with_context(|| format!("failed to read {}", path.display()))?;
        let sha256 = sha256_hex(&bytes);
        let extension = path
            .extension()
            .and_then(|ext| ext.to_str())
            .map(ToString::to_string);
        if let Some(ext) = &extension {
            *extensions.entry(ext.clone()).or_default() += 1;
        }
        if let Some(name) = path.file_name().and_then(|name| name.to_str())
            && is_manifest(name)
        {
            manifests.push(path.display().to_string());
        }
        let text = std::str::from_utf8(&bytes).ok();
        files.push(IndexedFile {
            path: normalize_path(path),
            extension,
            bytes: bytes.len() as u64,
            lines: text.map(|content| content.lines().count()),
            sha256,
            symbols: text.map(extract_symbols).unwrap_or_default(),
        });
    }

    files.sort_by(|a, b| a.path.cmp(&b.path));
    Ok(ProjectIndex {
        root: normalize_path(root),
        files,
        extensions,
        manifests,
    })
}

pub fn print_project_scan(path: impl AsRef<Path>) -> Result<()> {
    let index = build_index(path)?;
    heading("Project Scan");
    println!("files       {}", index.files.len());
    println!("root        {}", index.root);
    println!("\nTop extensions:");
    for (ext, count) in index.extensions.iter().take(12) {
        println!("{:<12} {}", ext, count);
    }
    println!("\nManifests:");
    for manifest in index.manifests.iter().take(12) {
        println!("{manifest}");
    }
    println!("\nSymbol Hints:");
    for file in index
        .files
        .iter()
        .filter(|file| !file.symbols.is_empty())
        .take(12)
    {
        println!("{} -> {}", file.path, file.symbols.join(", "));
    }
    Ok(())
}

pub async fn explain_project(
    patterns: &PatternEngine,
    config: &ForgeConfig,
    providers: &ProviderRegistry,
) -> Result<()> {
    let context = scan_context(".")?;
    let output = patterns.run("explain", &context, config, providers).await?;
    println!("{output}");
    Ok(())
}

pub async fn architecture(
    patterns: &PatternEngine,
    config: &ForgeConfig,
    providers: &ProviderRegistry,
) -> Result<()> {
    let context = scan_context(".")?;
    let output = patterns
        .run("architecture", &context, config, providers)
        .await?;
    println!("{output}");
    Ok(())
}

pub async fn generate_docs(
    patterns: &PatternEngine,
    config: &ForgeConfig,
    providers: &ProviderRegistry,
) -> Result<()> {
    let context = scan_context(".")?;
    let output = patterns
        .run("summarize", &context, config, providers)
        .await?;
    println!("{output}");
    Ok(())
}

pub fn find_dead_code(path: impl AsRef<Path>) -> Result<()> {
    heading("Dead Code Signals");
    for result in WalkBuilder::new(path)
        .hidden(false)
        .filter_entry(|entry| !is_generated_dir(entry.path()))
        .build()
    {
        let entry = result?;
        if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
            continue;
        }
        let path = entry.path();
        let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
            continue;
        };
        if !matches!(ext, "rs" | "ts" | "tsx" | "js" | "jsx" | "py") {
            continue;
        }
        let Ok(content) = std::fs::read_to_string(path) else {
            continue;
        };
        for (idx, line) in content.lines().enumerate() {
            if line.contains("TODO") || line.contains("dead_code") || line.contains("unused") {
                println!("{}:{} {}", path.display(), idx + 1, line.trim());
            }
        }
    }
    Ok(())
}

fn scan_context(path: impl AsRef<Path>) -> Result<String> {
    let index = build_index(path)?;
    let symbol_context = index
        .files
        .iter()
        .filter(|file| !file.symbols.is_empty())
        .take(40)
        .map(|file| format!("{}: {}", file.path, file.symbols.join(", ")))
        .collect::<Vec<_>>()
        .join("\n");
    Ok(format!(
        "Root: {}\nFiles: {}\nExtensions: {:?}\nManifests: {:?}\nSymbols:\n{}",
        index.root,
        index.files.len(),
        index.extensions,
        index.manifests,
        symbol_context
    ))
}

pub fn index_context(path: impl AsRef<Path>) -> Result<String> {
    scan_context(path)
}

fn is_manifest(name: &str) -> bool {
    matches!(
        name,
        "Cargo.toml"
            | "package.json"
            | "pyproject.toml"
            | "go.mod"
            | "requirements.txt"
            | "pom.xml"
            | "build.gradle"
            | "mix.exs"
    )
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

fn extract_symbols(content: &str) -> Vec<String> {
    let mut symbols = Vec::new();
    for line in content.lines() {
        let trimmed = line.trim_start();
        let prefixes = [
            "pub fn ",
            "fn ",
            "pub struct ",
            "struct ",
            "pub enum ",
            "enum ",
            "pub trait ",
            "trait ",
            "class ",
            "def ",
            "function ",
            "export function ",
            "interface ",
            "type ",
        ];
        for prefix in prefixes {
            if let Some(rest) = trimmed.strip_prefix(prefix) {
                let symbol = rest
                    .split(|ch: char| !ch.is_alphanumeric() && ch != '_' && ch != '-')
                    .next()
                    .unwrap_or_default();
                if !symbol.is_empty() {
                    symbols.push(format!("{}{}", prefix.trim_end(), symbol));
                }
                break;
            }
        }
        if symbols.len() >= 16 {
            break;
        }
    }
    symbols
}

fn sha256_hex(bytes: &[u8]) -> String {
    let digest = Sha256::digest(bytes);
    digest.iter().map(|byte| format!("{byte:02x}")).collect()
}

fn normalize_path(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
