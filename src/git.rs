/*
 * Copyright (c) 2026 Acme Corp. All rights reserved.
 */

use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct GitMetadata {
    pub created_year: Option<u32>,
    pub modified_year: Option<u32>,
    pub author_name: Option<String>,
    pub is_dirty: bool,
}

pub fn extract_git_metadata(file_path: &Path, enabled: bool) -> GitMetadata {
    if !enabled {
        return GitMetadata::default();
    }

    // Check if the file has uncommitted changes in git worktree
    let is_dirty = Command::new("git")
        .args([
            "status",
            "--porcelain",
            "--",
            file_path.to_str().unwrap_or(""),
        ])
        .output()
        .map(|out| !out.stdout.is_empty())
        .unwrap_or(false);

    // Get creation year (first commit date where file was added, or oldest commit)
    let created_year = Command::new("git")
        .args([
            "log",
            "--diff-filter=A",
            "--format=%ad",
            "--date=format:%Y",
            "-1",
            "--",
            file_path.to_str().unwrap_or(""),
        ])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() && !out.stdout.is_empty() {
                String::from_utf8(out.stdout)
                    .ok()
                    .and_then(|s| s.trim().parse::<u32>().ok())
            } else {
                None
            }
        })
        .or_else(|| {
            Command::new("git")
                .args([
                    "log",
                    "--format=%ad",
                    "--date=format:%Y",
                    "--",
                    file_path.to_str().unwrap_or(""),
                ])
                .output()
                .ok()
                .and_then(|out| {
                    if out.status.success() {
                        let text = String::from_utf8_lossy(&out.stdout);
                        text.lines().last().and_then(|l| l.trim().parse::<u32>().ok())
                    } else {
                        None
                    }
                })
        });

    // Get modified year (last commit year)
    let modified_year = Command::new("git")
        .args([
            "log",
            "--follow",
            "--format=%ad",
            "--date=format:%Y",
            "-1",
            "--",
            file_path.to_str().unwrap_or(""),
        ])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                String::from_utf8(out.stdout)
                    .ok()
                    .and_then(|s| s.trim().parse::<u32>().ok())
            } else {
                None
            }
        })
        .or(created_year);

    // Get initial author
    let author_name = Command::new("git")
        .args([
            "log",
            "--follow",
            "--format=%an",
            "--reverse",
            "-1",
            "--",
            file_path.to_str().unwrap_or(""),
        ])
        .output()
        .ok()
        .and_then(|out| {
            if out.status.success() {
                let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
                if s.is_empty() {
                    None
                } else {
                    Some(s)
                }
            } else {
                None
            }
        });

    GitMetadata {
        created_year,
        modified_year,
        author_name,
        is_dirty,
    }
}
