/*
 * Copyright (c) 2026 Acme Corp. All rights reserved.
 */

pub mod classifier;
pub mod config;
pub mod extractor;
pub mod formatter;
pub mod git;
pub mod walker;

pub use classifier::Classifier;
pub use config::{CommentStyle, Config, HeaderConfig, LanguageRule};
pub use extractor::Extractor;
pub use formatter::{FormatResult, Formatter};
pub use git::{extract_git_metadata, GitMetadata};
pub use walker::discover_files;
