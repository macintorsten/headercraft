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
use std::sync::Mutex;

static COMMAND_CACHE: Mutex<Option<HashMap<String, Option<String>>>> = Mutex::new(None);

/// Clears the global command execution cache.
pub fn clear_command_cache() {
    if let Ok(mut guard) = COMMAND_CACHE.lock() {
        if let Some(ref mut cache) = *guard {
            cache.clear();
        }
    }
}

/// Executes a shell command, caching the result by `cmd_str`.
/// If `cmd_str` does not reference `{file}`, subsequent invocations across files hit the cache.
pub fn execute_command_cached(cmd_str: &str, file_path: Option<&str>) -> Option<String> {
    // 1. Check cache
    if let Ok(guard) = COMMAND_CACHE.lock() {
        if let Some(ref cache) = *guard {
            if let Some(cached_result) = cache.get(cmd_str) {
                return cached_result.clone();
            }
        }
    }

    // 2. Execute process
    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("cmd");
    #[cfg(target_os = "windows")]
    command.args(["/C", cmd_str]);

    #[cfg(not(target_os = "windows"))]
    let mut command = std::process::Command::new("sh");
    #[cfg(not(target_os = "windows"))]
    command.args(["-c", cmd_str]);

    if let Some(fp) = file_path {
        command.env("HC_FILE", fp);
        command.env("FILE", fp);
    }

    let cmd_output = command.output();

    let result = if let Ok(out) = cmd_output {
        let stdout_text = String::from_utf8_lossy(&out.stdout).trim().to_string();
        let val = if !stdout_text.is_empty() {
            stdout_text
        } else if out.status.success() {
            String::from_utf8_lossy(&out.stderr).trim().to_string()
        } else {
            String::new()
        };

        if !val.is_empty() {
            Some(val)
        } else {
            None
        }
    } else {
        None
    };

    // 3. Store in cache
    if let Ok(mut guard) = COMMAND_CACHE.lock() {
        let cache = guard.get_or_insert_with(HashMap::new);
        cache.insert(cmd_str.to_string(), result.clone());
    }

    result
}

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
            git_map.insert("last_commit_year".to_string(), json!(y));
        }
        if let Some(a) = git_meta.author_name {
            git_map.insert("author".to_string(), json!(a));
        }
        git_map.insert("is_dirty".to_string(), json!(git_meta.is_dirty));

        let mut fs_map = serde_json::Map::new();
        fs_map.insert("filename".to_string(), json!(filename));
        fs_map.insert("stem".to_string(), json!(stem));
        fs_map.insert("extension".to_string(), json!(extension));
        fs_map.insert("dir".to_string(), json!(dir));
        if let Some(y) = mtime_year {
            fs_map.insert("mtime_year".to_string(), json!(y));
        }

        // 4. Custom command outputs with command-based caching
        let mut cmd_map = serde_json::Map::new();
        let path_str = path.display().to_string();
        for (key, cmd_template) in &config.commands {
            let cmd_str = cmd_template
                .replace("{file}", &path_str)
                .replace("{path}", &path_str)
                .replace("{filename}", filename)
                .replace("{stem}", stem)
                .replace("{dir}", dir);

            if let Some(val) = execute_command_cached(&cmd_str, Some(&path_str)) {
                cmd_map.insert(key.clone(), json!(val));
            }
        }

        // Build comprehensive JSON context for MiniJinja
        json!({
            "vars": vars,
            "git": Value::Object(git_map),
            "fs": Value::Object(fs_map),
            "cmd": Value::Object(cmd_map),
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

    #[test]
    fn test_extract_command_with_file_placeholder() {
        let mut config = Config::default();
        config
            .commands
            .insert("inspect".into(), "echo Target is {file}".into());

        let ctx = Extractor::build_context(Path::new("scripts/test.sh"), "", &config);
        let cmd = ctx.get("cmd").unwrap();
        assert_eq!(cmd.get("inspect").unwrap(), "Target is scripts/test.sh");
    }

    #[test]
    fn test_command_caching_behavior() {
        clear_command_cache();

        // 1. Static command: cache hit across different files
        let static_cmd = "echo static_output_123";
        let out1 = execute_command_cached(static_cmd, None);
        assert_eq!(out1.as_deref(), Some("static_output_123"));

        // Subsequent call with same command hits cache
        let out2 = execute_command_cached(static_cmd, None);
        assert_eq!(out2.as_deref(), Some("static_output_123"));

        // 2. File-specific command: different command string = cache miss / independent entries
        let file_cmd_a = "echo file_a";
        let file_cmd_b = "echo file_b";
        let out_a = execute_command_cached(file_cmd_a, Some("a.sh"));
        let out_b = execute_command_cached(file_cmd_b, Some("b.sh"));
        assert_eq!(out_a.as_deref(), Some("file_a"));
        assert_eq!(out_b.as_deref(), Some("file_b"));

        // Same file-specific command again = cache hit
        let out_a_cached = execute_command_cached(file_cmd_a, Some("a.sh"));
        assert_eq!(out_a_cached.as_deref(), Some("file_a"));

        // 3. Clear cache
        clear_command_cache();
    }
}
