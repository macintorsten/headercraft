/*
 * Copyright (c) 2026 Acme Corp. All rights reserved.
 */

use crate::config::Config;
use glob::Pattern;
use ignore::WalkBuilder;
use std::path::{Path, PathBuf};

pub fn discover_files(files: &[PathBuf], config: &Config) -> Vec<PathBuf> {
    if !files.is_empty() {
        let mut resolved = Vec::new();
        for f in files {
            if f.is_file() {
                resolved.push(f.clone());
            } else if f.is_dir() {
                resolved.extend(walk_directory(f, config));
            }
        }
        return resolved;
    }

    walk_directory(Path::new("."), config)
}

fn walk_directory(dir: &Path, config: &Config) -> Vec<PathBuf> {
    let mut results = Vec::new();

    let include_patterns: Vec<Pattern> = config
        .files
        .includes
        .iter()
        .filter_map(|pat| Pattern::new(pat).ok())
        .collect();

    let exclude_patterns: Vec<Pattern> = config
        .files
        .excludes
        .iter()
        .filter_map(|pat| Pattern::new(pat).ok())
        .collect();

    let walker = WalkBuilder::new(dir)
        .standard_filters(true) // respects .gitignore, hidden files
        .build();

    for entry in walker.flatten() {
        let path = entry.path();
        if !path.is_file() {
            continue;
        }

        let relative_path = path.strip_prefix("./").unwrap_or(path);
        let path_str = relative_path.to_str().unwrap_or("");

        // Check exclusions
        let is_excluded = exclude_patterns.iter().any(|pat| pat.matches(path_str));
        if is_excluded {
            continue;
        }

        // Check inclusions
        let is_included = if include_patterns.is_empty() {
            true
        } else {
            include_patterns.iter().any(|pat| pat.matches(path_str))
        };

        if is_included {
            results.push(relative_path.to_path_buf());
        }
    }

    results
}
