/*
 * Copyright (c) 2026 Acme Corp. All rights reserved.
 */

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    #[serde(default)]
    pub header: HeaderConfig,
    #[serde(default)]
    pub props: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub git: GitConfig,
    #[serde(default)]
    pub files: FilesConfig,
    #[serde(default)]
    pub rules: Vec<LanguageRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderConfig {
    pub template: String,
    pub extract_regex: Option<String>,
    #[serde(default = "default_keywords")]
    pub keywords: Vec<String>,
}

fn default_keywords() -> Vec<String> {
    vec![
        "copyright".to_string(),
        "license".to_string(),
        "spdx".to_string(),
    ]
}

impl Default for HeaderConfig {
    fn default() -> Self {
        Self {
            template: "{% set start_year = vars.start_year | default(git.created_year) | default(now.year) %}Copyright (c) {{ start_year }}{% if start_year | string != now.year | string %}-{{ now.year }}{% endif %} {{ vars.owner | default(props.owner) | default('Acme Corp') }}. All rights reserved.".to_string(),
            extract_regex: Some(r"(?i)copyright\s+(?:\(c\)\s+)?(?P<start_year>\d{4})(?:-(?P<end_year>\d{4}))?\s+(?P<owner>[^\n\.\,]+)".to_string()),
            keywords: default_keywords(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GitConfig {
    #[serde(default = "default_true")]
    pub enabled: bool,
}

impl Default for GitConfig {
    fn default() -> Self {
        Self { enabled: true }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FilesConfig {
    #[serde(default)]
    pub includes: Vec<String>,
    #[serde(default = "default_excludes")]
    pub excludes: Vec<String>,
}

fn default_excludes() -> Vec<String> {
    vec![
        "target/**".to_string(),
        "build/**".to_string(),
        ".cache/**".to_string(),
        ".git/**".to_string(),
        "node_modules/**".to_string(),
        "vendor/**".to_string(),
        "**/*.min.js".to_string(),
        "**/*.min.css".to_string(),
    ]
}

impl Default for FilesConfig {
    fn default() -> Self {
        Self {
            includes: vec!["**/*".to_string()],
            excludes: default_excludes(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommentStyle {
    Block,
    Line,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageRule {
    pub name: String,
    #[serde(default)]
    pub extensions: Vec<String>,
    #[serde(default)]
    pub filenames: Vec<String>,
    #[serde(default)]
    pub content_patterns: Vec<String>,
    pub style: CommentStyle,
    pub comment_start: Option<String>,
    #[serde(default)]
    pub comment_prefix: String,
    pub comment_end: Option<String>,
    #[serde(default = "default_true")]
    pub preserve_shebang: bool,
    #[serde(default = "default_true")]
    pub preserve_xml_prolog: bool,
}

fn default_true() -> bool {
    true
}

impl Config {
    pub fn load_from_file(path: &Path) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;
        config.merge_default_rules_if_empty();
        Ok(config)
    }

    pub fn load_or_default(path: Option<&Path>) -> Self {
        if let Some(p) = path {
            if let Ok(cfg) = Self::load_from_file(p) {
                return cfg;
            }
        } else {
            for default_name in &["headercraft.toml", ".headercraft.toml", "licenserc.toml"] {
                let p = Path::new(default_name);
                if p.is_file() {
                    if let Ok(cfg) = Self::load_from_file(p) {
                        return cfg;
                    }
                }
            }
        }

        let mut cfg = Config {
            header: HeaderConfig::default(),
            props: HashMap::new(),
            git: GitConfig::default(),
            files: FilesConfig::default(),
            rules: Vec::new(),
        };
        cfg.merge_default_rules_if_empty();
        cfg
    }

    pub fn merge_default_rules_if_empty(&mut self) {
        if self.rules.is_empty() {
            self.rules = default_language_rules();
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        let mut cfg = Self {
            header: HeaderConfig::default(),
            props: HashMap::new(),
            git: GitConfig::default(),
            files: FilesConfig::default(),
            rules: Vec::new(),
        };
        cfg.merge_default_rules_if_empty();
        cfg
    }
}

pub fn default_language_rules() -> Vec<LanguageRule> {
    vec![
        LanguageRule {
            name: "c_style_block".to_string(),
            extensions: vec![
                "c".into(),
                "cpp".into(),
                "cc".into(),
                "h".into(),
                "hpp".into(),
                "java".into(),
                "js".into(),
                "jsx".into(),
                "ts".into(),
                "tsx".into(),
                "cs".into(),
                "go".into(),
                "rs".into(),
                "scala".into(),
                "kt".into(),
                "css".into(),
                "scss".into(),
            ],
            filenames: vec![],
            content_patterns: vec![],
            style: CommentStyle::Block,
            comment_start: Some("/*".into()),
            comment_prefix: " * ".into(),
            comment_end: Some(" */".into()),
            preserve_shebang: true,
            preserve_xml_prolog: false,
        },
        LanguageRule {
            name: "hash_line".to_string(),
            extensions: vec![
                "py".into(),
                "sh".into(),
                "bash".into(),
                "zsh".into(),
                "ps1".into(),
                "yaml".into(),
                "yml".into(),
                "toml".into(),
                "rb".into(),
                "r".into(),
                "pl".into(),
            ],
            filenames: vec!["Dockerfile".into(), "Makefile".into(), "Gemfile".into()],
            content_patterns: vec!["^#!.*".into()],
            style: CommentStyle::Line,
            comment_start: None,
            comment_prefix: "# ".into(),
            comment_end: None,
            preserve_shebang: true,
            preserve_xml_prolog: false,
        },
        LanguageRule {
            name: "xml_block".to_string(),
            extensions: vec![
                "xml".into(),
                "html".into(),
                "htm".into(),
                "svg".into(),
                "pom".into(),
                "xaml".into(),
            ],
            filenames: vec![],
            content_patterns: vec!["<\\?xml.*".into(), "<!DOCTYPE.*".into()],
            style: CommentStyle::Block,
            comment_start: Some("<!--".into()),
            comment_prefix: "  ".into(),
            comment_end: Some("-->".into()),
            preserve_shebang: false,
            preserve_xml_prolog: true,
        },
        LanguageRule {
            name: "sql_double_dash".to_string(),
            extensions: vec!["sql".into(), "lua".into(), "hs".into()],
            filenames: vec![],
            content_patterns: vec![],
            style: CommentStyle::Line,
            comment_start: None,
            comment_prefix: "-- ".into(),
            comment_end: None,
            preserve_shebang: false,
            preserve_xml_prolog: false,
        },
    ]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert!(config.git.enabled);
        assert!(!config.rules.is_empty());
        assert!(config.header.extract_regex.is_some());
    }

    #[test]
    fn test_parse_custom_config() {
        let toml_str = r#"
[header]
template = "Custom Header: {{ props.author }}"
extract_regex = 'Author:\s+(?P<author>\w+)'

[props]
author = "Alice"
team = "Core"

[git]
enabled = false

[files]
includes = ["src/**"]
excludes = ["vendor/**"]

[[rules]]
name = "custom_lang"
extensions = ["xyz"]
style = "line"
comment_prefix = "// "
"#;
        let config: Config = toml::from_str(toml_str).unwrap();
        assert_eq!(config.header.template, "Custom Header: {{ props.author }}");
        assert_eq!(
            config.props.get("author").and_then(|v| v.as_str()),
            Some("Alice")
        );
        assert!(!config.git.enabled);
        assert_eq!(config.files.includes, vec!["src/**"]);
        assert_eq!(config.rules.len(), 1);
        assert_eq!(config.rules[0].name, "custom_lang");
    }
}
