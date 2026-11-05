/*
 * Copyright (c) 2026 Acme Corp. All rights reserved.
 */

use chrono::Datelike;
use std::path::Path;
use std::process::Command;

#[derive(Debug, Clone, Default)]
pub struct GitMetadata {
    pub created_year: Option<u32>,
    pub modified_year: Option<u32>,
    pub author_name: Option<String>,
}

pub fn extract_git_metadata(file_path: &Path, enabled: bool) -> GitMetadata {
    if !enabled {
        return GitMetadata::default();
    }

    let current_year = chrono::Utc::now().year() as u32;

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

    // Get creation year (first commit date)
    let created_year = Command::new("git")
        .args([
            "log",
            "--follow",
            "--format=%ad",
            "--date=format:%Y",
            "--reverse",
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
        });

    // Get modified year
    let modified_year = if is_dirty {
        Some(current_year)
    } else {
        Command::new("git")
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
            .or(created_year)
    };

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
    }
}
