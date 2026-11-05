/*
 * Copyright (c) 2018-2026 Jane Doe. All rights reserved.
 */

use crate::config::Config;
use crate::git::{extract_git_metadata, GitMetadata};
use chrono::Datelike;
use regex::Regex;
use serde_json::{json, Value};
use std::collections::HashMap;
use std::path::Path;

pub struct Extractor;

impl Extractor {
    pub fn build_context(path: &Path, content: &str, config: &Config) -> Value {
        let now = chrono::Utc::now();
        let now_year = now.year();
        let now_date = now.format("%Y-%m-%d").to_string();
        let now_iso = now.to_rfc3339();

        // 1. Filesystem attributes
        let filename = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        let stem = path.file_stem().and_then(|s| s.to_str()).unwrap_or("");
        let extension = path.extension().and_then(|e| e.to_str()).unwrap_or("");
        let dir = path.parent().and_then(|p| p.to_str()).unwrap_or(".");
        let mtime_year = std::fs::metadata(path)
            .ok()
            .and_then(|m| m.modified().ok())
            .map(|t| {
                let dt: chrono::DateTime<chrono::Utc> = t.into();
                dt.year()
            });

        // 2. Git attributes
        let git_meta: GitMetadata = extract_git_metadata(path, config.git.enabled);

        // 3. Extract named capture groups from existing content
        let mut vars = HashMap::new();
        if let Some(ref pattern) = config.header.extract_regex {
            if let Ok(re) = Regex::new(pattern) {
                if let Some(caps) = re.captures(content) {
                    for name in re.capture_names().flatten() {
                        if let Some(m) = caps.name(name) {
                            vars.insert(name.to_string(), m.as_str().trim().to_string());
                        }
                    }
                }
            }
        }

        let mut git_map = serde_json::Map::new();
        if let Some(y) = git_meta.created_year {
            git_map.insert("created_year".to_string(), json!(y));
        }
        if let Some(y) = git_meta.modified_year {
            git_map.insert("modified_year".to_string(), json!(y));
        }
        if let Some(a) = git_meta.author_name {
            git_map.insert("author".to_string(), json!(a));
        }

        let mut fs_map = serde_json::Map::new();
        fs_map.insert("filename".to_string(), json!(filename));
        fs_map.insert("stem".to_string(), json!(stem));
        fs_map.insert("extension".to_string(), json!(extension));
        fs_map.insert("dir".to_string(), json!(dir));
        if let Some(y) = mtime_year {
            fs_map.insert("mtime_year".to_string(), json!(y));
        }

        // Build comprehensive JSON context for MiniJinja
        json!({
            "vars": vars,
            "git": Value::Object(git_map),
            "fs": Value::Object(fs_map),
            "now": {
                "year": now_year,
                "date": now_date,
                "iso": now_iso,
            },
            "props": config.props,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_regex_named_captures() {
        let config = Config::default();
        let content =
            "/*\n * Copyright (c) 2018-2021 Jane Doe. All rights reserved.\n */\npublic class A {}";

        let ctx = Extractor::build_context(Path::new("src/A.java"), content, &config);

        let vars = ctx.get("vars").unwrap();
        assert_eq!(vars.get("start_year").unwrap(), "2018");
        assert_eq!(vars.get("end_year").unwrap(), "2021");
        assert_eq!(vars.get("owner").unwrap(), "Jane Doe");
    }

    #[test]
    fn test_extract_fs_and_now_metadata() {
        let config = Config::default();
        let content = "public class Hello {}";
        let ctx = Extractor::build_context(Path::new("pkg/sub/Hello.java"), content, &config);

        let fs = ctx.get("fs").unwrap();
        assert_eq!(fs.get("filename").unwrap(), "Hello.java");
        assert_eq!(fs.get("stem").unwrap(), "Hello");
        assert_eq!(fs.get("extension").unwrap(), "java");

        let now = ctx.get("now").unwrap();
        let current_year = chrono::Utc::now().year();
        assert_eq!(now.get("year").unwrap().as_i64(), Some(current_year as i64));

        let vars = ctx.get("vars").unwrap();
        assert!(vars.as_object().unwrap().is_empty());
    }
}
