/*
 * Copyright (c) 2015-2026 Old Corp. All rights reserved.
 */

use crate::config::{CommentStyle, Config, LanguageRule};
use crate::extractor::Extractor;
use minijinja::Environment;
use similar::TextDiff;
use std::path::Path;

pub struct Formatter;

#[derive(Debug, PartialEq, Eq)]
pub enum FormatResult {
    Unchanged,
    Changed { new_content: String, diff: String },
}

impl Formatter {
    pub fn render_header(
        config: &Config,
        rule: &LanguageRule,
        path: &Path,
        content: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let mut env = Environment::new();
        env.add_template("header", &config.header.template)?;
        let tmpl = env.get_template("header")?;

        let ctx = Extractor::build_context(path, content, config);
        let rendered_text = tmpl.render(ctx)?;

        Ok(Self::wrap_comment(rule, &rendered_text))
    }

    pub fn wrap_comment(rule: &LanguageRule, text: &str) -> String {
        let mut out = String::new();
        let trimmed = text.trim();

        match rule.style {
            CommentStyle::Block => {
                if let Some(ref start) = rule.comment_start {
                    out.push_str(start);
                    out.push('\n');
                }
                for line in trimmed.lines() {
                    out.push_str(&rule.comment_prefix);
                    out.push_str(line);
                    out.push('\n');
                }
                if let Some(ref end) = rule.comment_end {
                    out.push_str(end);
                    out.push('\n');
                }
            }
            CommentStyle::Line => {
                for line in trimmed.lines() {
                    out.push_str(&rule.comment_prefix);
                    out.push_str(line);
                    out.push('\n');
                }
            }
        }

        out
    }

    pub fn format_file(
        path: &Path,
        content: &str,
        rule: &LanguageRule,
        config: &Config,
    ) -> Result<FormatResult, Box<dyn std::error::Error>> {
        let canonical_header = Self::render_header(config, rule, path, content)?;

        let lines: Vec<&str> = content.lines().collect();

        // 1. Detect if an existing header is present in the first ~30 lines
        let existing_header_range = Self::find_existing_header_range(&lines, rule, config);

        let new_content = if let Some((start_idx, end_idx)) = existing_header_range {
            // Replace existing header range
            let mut result = String::new();
            for line in lines.iter().take(start_idx) {
                result.push_str(line);
                result.push('\n');
            }
            result.push_str(&canonical_header);
            for line in lines.iter().skip(end_idx) {
                result.push_str(line);
                result.push('\n');
            }
            // Preserve original trailing newline if any
            if !content.ends_with('\n') && result.ends_with('\n') {
                result.pop();
            }
            result
        } else {
            // Insert new header
            let insert_idx = Self::find_insertion_index(&lines, rule);
            let mut result = String::new();
            for line in lines.iter().take(insert_idx) {
                result.push_str(line);
                result.push('\n');
            }
            result.push_str(&canonical_header);
            // Add a separating newline if code immediately follows
            if insert_idx < lines.len() && !lines[insert_idx].trim().is_empty() {
                result.push('\n');
            }
            for line in lines.iter().skip(insert_idx) {
                result.push_str(line);
                result.push('\n');
            }
            if !content.ends_with('\n') && result.ends_with('\n') {
                result.pop();
            }
            result
        };

        if new_content == content {
            Ok(FormatResult::Unchanged)
        } else {
            let diff = Self::generate_diff(path, content, &new_content);
            Ok(FormatResult::Changed { new_content, diff })
        }
    }

    fn find_insertion_index(lines: &[&str], rule: &LanguageRule) -> usize {
        if lines.is_empty() {
            return 0;
        }

        let mut idx = 0;

        // Skip shebang if rule preserves it
        if rule.preserve_shebang && lines[0].starts_with("#!") {
            idx += 1;
        }

        // Skip XML prolog if rule preserves it
        if rule.preserve_xml_prolog && lines[0].starts_with("<?xml") {
            idx += 1;
        }

        // Skip Python encoding declarations
        if idx < lines.len()
            && (lines[idx].contains("-*- coding:") || lines[idx].contains("coding="))
        {
            idx += 1;
        }

        idx
    }

    fn find_existing_header_range(
        lines: &[&str],
        rule: &LanguageRule,
        config: &Config,
    ) -> Option<(usize, usize)> {
        if lines.is_empty() {
            return None;
        }

        let max_search = lines.len().min(40);
        let search_text = lines[0..max_search].join("\n").to_lowercase();

        // Check if any recognition keyword is present
        let has_keyword = config
            .header
            .keywords
            .iter()
            .any(|k| search_text.contains(&k.to_lowercase()));
        if !has_keyword {
            return None;
        }

        match rule.style {
            CommentStyle::Block => {
                let start_token = rule.comment_start.as_deref().unwrap_or("/*").trim();
                let end_token = rule.comment_end.as_deref().unwrap_or("*/").trim();

                let mut in_block = false;
                let mut start_idx = None;

                for (i, line) in lines[0..max_search].iter().enumerate() {
                    let trimmed = line.trim();
                    if !in_block && trimmed.starts_with(start_token) {
                        in_block = true;
                        start_idx = Some(i);
                    }
                    if in_block && trimmed.ends_with(end_token) {
                        if let Some(s) = start_idx {
                            let block_text = lines[s..=i].join("\n").to_lowercase();
                            if config
                                .header
                                .keywords
                                .iter()
                                .any(|k| block_text.contains(&k.to_lowercase()))
                            {
                                return Some((s, i + 1));
                            }
                        }
                        in_block = false;
                        start_idx = None;
                    }
                }
            }
            CommentStyle::Line => {
                let prefix = rule.comment_prefix.trim();
                let mut in_comment = false;
                let mut start_idx = None;

                for (i, line) in lines[0..max_search].iter().enumerate() {
                    let trimmed = line.trim();
                    // Ignore shebang
                    if i == 0 && trimmed.starts_with("#!") {
                        continue;
                    }
                    if trimmed.starts_with(prefix) {
                        if !in_comment {
                            in_comment = true;
                            start_idx = Some(i);
                        }
                    } else if in_comment {
                        if let Some(s) = start_idx {
                            let block_text = lines[s..i].join("\n").to_lowercase();
                            if config
                                .header
                                .keywords
                                .iter()
                                .any(|k| block_text.contains(&k.to_lowercase()))
                            {
                                return Some((s, i));
                            }
                        }
                        in_comment = false;
                        start_idx = None;
                    }
                }

                if in_comment {
                    if let Some(s) = start_idx {
                        let block_text = lines[s..max_search].join("\n").to_lowercase();
                        if config
                            .header
                            .keywords
                            .iter()
                            .any(|k| block_text.contains(&k.to_lowercase()))
                        {
                            return Some((s, max_search));
                        }
                    }
                }
            }
        }

        None
    }

    pub fn generate_diff(path: &Path, old: &str, new: &str) -> String {
        let diff = TextDiff::from_lines(old, new);
        let path_str = path.display().to_string();
        diff.unified_diff()
            .header(&format!("a/{}", path_str), &format!("b/{}", path_str))
            .to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_language_rules;

    #[test]
    fn test_shebang_preservation() {
        let rules = default_language_rules();
        let py_rule = rules.iter().find(|r| r.name == "hash_line").unwrap();
        let config = Config::default();

        let initial = "#!/usr/bin/env python3\nprint('hello')\n";
        let res = Formatter::format_file(Path::new("test.py"), initial, py_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                let lines: Vec<&str> = new_content.lines().collect();
                assert_eq!(lines[0], "#!/usr/bin/env python3");
                assert!(lines[1].starts_with("# Copyright"));
            }
            _ => panic!("Expected Changed"),
        }
    }

    #[test]
    fn test_existing_header_replacement() {
        let rules = default_language_rules();
        let java_rule = rules.iter().find(|r| r.name == "c_style_block").unwrap();
        let config = Config::default();

        let initial = "/*\n * Copyright 2015 Old Corp.\n */\npublic class Foo {}\n";
        let res =
            Formatter::format_file(Path::new("Foo.java"), initial, java_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                // Verifies that start_year (2015) and owner (Old Corp) are extracted and reused in updated template!
                assert!(new_content.contains("2015-"));
                assert!(new_content.contains("Old Corp"));
                assert!(new_content.contains("All rights reserved."));
                assert!(new_content.contains("public class Foo {}"));
            }
            _ => panic!("Expected Changed"),
        }
    }

    #[test]
    fn test_xml_prolog_preservation() {
        let rules = default_language_rules();
        let xml_rule = rules.iter().find(|r| r.name == "xml_block").unwrap();
        let config = Config::default();

        let initial = "<?xml version=\"1.0\" encoding=\"UTF-8\"?>\n<root><child/></root>\n";
        let res =
            Formatter::format_file(Path::new("data.xml"), initial, xml_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                let lines: Vec<&str> = new_content.lines().collect();
                assert_eq!(lines[0], "<?xml version=\"1.0\" encoding=\"UTF-8\"?>");
                assert_eq!(lines[1], "<!--");
                assert!(lines[2].contains("Copyright (c)"));
                assert_eq!(lines[3], "-->");
            }
            _ => panic!("Expected Changed"),
        }
    }

    #[test]
    fn test_already_valid_header_unchanged() {
        let rules = default_language_rules();
        let py_rule = rules.iter().find(|r| r.name == "hash_line").unwrap();
        let config = Config::default();

        let initial = "#!/usr/bin/env python3\nprint('hello')\n";
        let first_res =
            Formatter::format_file(Path::new("test.py"), initial, py_rule, &config).unwrap();
        let formatted = match first_res {
            FormatResult::Changed { new_content, .. } => new_content,
            _ => panic!("Expected Changed"),
        };

        // Formatting again should be idempotent
        let second_res =
            Formatter::format_file(Path::new("test.py"), &formatted, py_rule, &config).unwrap();
        assert!(matches!(second_res, FormatResult::Unchanged));
    }
}
