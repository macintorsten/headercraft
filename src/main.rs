/*
 * Copyright (c) 2026 Acme Corp. All rights reserved.
 */

use clap::{Parser, Subcommand, ValueEnum};
use headercraft::classifier::Classifier;
use headercraft::config::Config;
use headercraft::formatter::{FormatResult, Formatter};
use headercraft::walker::discover_files;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use std::process::ExitCode;

#[derive(Parser)]
#[command(
    name = "headercraft",
    version = "0.1.0",
    about = "Pluggable file header validator and formatter"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(ValueEnum, Clone, Copy, Debug, PartialEq, Eq)]
enum OutputFormat {
    Human,
    Gcc,
    Sarif,
}

#[derive(Subcommand)]
enum Commands {
    /// Check files for valid headers
    Check {
        /// Output format (human, gcc for hk, sarif)
        #[arg(short, long, default_value = "human")]
        format: OutputFormat,

        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Specific files or directories to check
        files: Vec<PathBuf>,
    },

    /// Format and fix headers in-place
    Fix {
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Specific files or directories to format
        files: Vec<PathBuf>,
    },

    /// Show unified diff of changes without modifying files
    Diff {
        /// Path to configuration file
        #[arg(short, long)]
        config: Option<PathBuf>,

        /// Specific files or directories to diff
        files: Vec<PathBuf>,
    },

    /// Generate starter headercraft.toml configuration
    Init {
        #[arg(short, long, default_value = "headercraft.toml")]
        output: PathBuf,
    },
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init { output } => {
            if output.exists() {
                eprintln!("Error: {:?} already exists.", output);
                return ExitCode::from(1);
            }
            let default_toml = r#"# HeaderCraft Configuration
[header]
# Jinja template for canonical header
template = """
{% set start_year = vars.start_year | default(git.created_year) | default(now.year) -%}
Copyright (c) {{ start_year }}{% if start_year | string != now.year | string %}-{{ now.year }}{% endif %} {{ vars.owner | default(props.owner) | default('Acme Corp') }}. All rights reserved.
"""

# Regex with named capture groups to extract existing metadata from file headers
extract_regex = '(?i)copyright\s+(?:\(c\)\s+)?(?P<start_year>\d{4})(?:-(?P<end_year>\d{4}))?\s+(?P<owner>[^\n\.\,]+)'

[props]
owner = "Acme Corp"
license = "Apache-2.0"

[git]
enabled = true

[files]
includes = ["**/*"]
excludes = ["target/**", "build/**", ".cache/**", ".git/**", "node_modules/**"]
"#;
            if let Err(e) = fs::write(&output, default_toml) {
                eprintln!("Error writing {:?}: {}", output, e);
                return ExitCode::from(1);
            }
            println!("Created {:?} successfully.", output);
            ExitCode::SUCCESS
        }

        Commands::Diff { config, files } => {
            let cfg = Config::load_or_default(config.as_deref());
            let target_files = discover_files(&files, &cfg);
            let classifier = Classifier::new(&cfg.rules);
            let mut has_diff = false;

            for path in target_files {
                let Ok(content) = fs::read_to_string(&path) else {
                    continue;
                };
                let Some(rule) = classifier.classify(&path, &content) else {
                    continue;
                };

                if !rule.fixable {
                    continue;
                }

                if let Ok(FormatResult::Changed { diff, .. }) =
                    Formatter::format_file(&path, &content, rule, &cfg)
                {
                    print!("{}", diff);
                    has_diff = true;
                }
            }

            if has_diff {
                ExitCode::from(1)
            } else {
                ExitCode::SUCCESS
            }
        }

        Commands::Fix { config, files } => {
            let cfg = Config::load_or_default(config.as_deref());
            let target_files = discover_files(&files, &cfg);
            let classifier = Classifier::new(&cfg.rules);
            let mut modified_count = 0;
            let mut manual_fix_count = 0;

            for path in target_files {
                let Ok(content) = fs::read_to_string(&path) else {
                    continue;
                };
                let Some(rule) = classifier.classify(&path, &content) else {
                    continue;
                };

                if let Ok(FormatResult::Changed { new_content, .. }) =
                    Formatter::format_file(&path, &content, rule, &cfg)
                {
                    if !rule.fixable {
                        eprintln!(
                            "Manual fix required: {} (rule '{}' is check-only)",
                            path.display(),
                            rule.name
                        );
                        manual_fix_count += 1;
                        continue;
                    }
                    if let Err(e) = fs::write(&path, new_content) {
                        eprintln!("Error writing {}: {}", path.display(), e);
                    } else {
                        println!("Updated header: {}", path.display());
                        modified_count += 1;
                    }
                }
            }

            if manual_fix_count > 0 {
                println!(
                    "Fixed {} file(s) ({} file(s) require manual fix).",
                    modified_count, manual_fix_count
                );
                ExitCode::from(1)
            } else {
                println!("Fixed {} file(s).", modified_count);
                ExitCode::SUCCESS
            }
        }

        Commands::Check {
            format,
            config,
            files,
        } => {
            let cfg = Config::load_or_default(config.as_deref());
            let target_files = discover_files(&files, &cfg);
            let classifier = Classifier::new(&cfg.rules);
            let mut violations: Vec<(PathBuf, String, String, bool)> = Vec::new();

            for path in target_files {
                let Ok(content) = fs::read_to_string(&path) else {
                    continue;
                };
                let Some(rule) = classifier.classify(&path, &content) else {
                    continue;
                };

                if let Ok(FormatResult::Changed { diff, .. }) =
                    Formatter::format_file(&path, &content, rule, &cfg)
                {
                    violations.push((path, diff, rule.name.clone(), rule.fixable));
                }
            }

            if violations.is_empty() {
                if format == OutputFormat::Human {
                    println!("All files have valid headers.");
                }
                return ExitCode::SUCCESS;
            }

            match format {
                OutputFormat::Gcc => {
                    for (path, _, rule_name, fixable) in &violations {
                        let note = if *fixable {
                            ""
                        } else {
                            " (manual fix required)"
                        };
                        println!(
                            "{}:1:1: error: Missing or outdated file header{} [{}]",
                            path.display(),
                            note,
                            rule_name
                        );
                    }
                }
                OutputFormat::Human => {
                    eprintln!("Header violations found in {} file(s):\n", violations.len());
                    for (path, diff, rule_name, fixable) in &violations {
                        let fix_msg = if *fixable {
                            "auto-fixable"
                        } else {
                            "manual fix required"
                        };
                        eprintln!(
                            "--- {} [rule: {}, {}] ---",
                            path.display(),
                            rule_name,
                            fix_msg
                        );
                        eprintln!("{}", diff);
                    }
                }
                OutputFormat::Sarif => {
                    let results: Vec<_> = violations
                        .iter()
                        .map(|(p, _, rule_name, fixable)| {
                            let msg = if *fixable {
                                "Missing or outdated file header"
                            } else {
                                "Missing or outdated file header (manual fix required)"
                            };
                            json!({
                                "ruleId": rule_name,
                                "message": { "text": msg },
                                "locations": [{
                                    "physicalLocation": {
                                        "artifactLocation": { "uri": p.to_string_lossy() },
                                        "region": { "startLine": 1, "startColumn": 1 }
                                    }
                                }]
                            })
                        })
                        .collect();

                    let sarif = json!({
                        "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
                        "version": "2.1.0",
                        "runs": [{
                            "tool": {
                                "driver": {
                                    "name": "headercraft",
                                    "version": "0.1.0"
                                }
                            },
                            "results": results
                        }]
                    });
                    println!("{}", serde_json::to_string_pretty(&sarif).unwrap());
                }
            }

            ExitCode::from(1)
        }
    }
}
