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
        eol: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let template_text = rule.template.as_deref().unwrap_or(&config.header.template);
        let mut env = Environment::new();
        env.add_template("header", template_text)?;
        let tmpl = env.get_template("header")?;

        let ctx = Extractor::build_context(path, content, config);
        let rendered_text = tmpl.render(ctx)?;

        Ok(Self::wrap_comment(rule, &rendered_text, eol))
    }

    pub fn wrap_comment(rule: &LanguageRule, text: &str, eol: &str) -> String {
        let mut out = String::new();
        let trimmed = text.trim();

        match rule.style {
            CommentStyle::Block => {
                if let Some(ref start) = rule.comment_start {
                    out.push_str(start);
                    out.push_str(eol);
                }
                for line in trimmed.lines() {
                    out.push_str(&rule.comment_prefix);
                    out.push_str(line);
                    out.push_str(eol);
                }
                if let Some(ref end) = rule.comment_end {
                    out.push_str(end);
                    out.push_str(eol);
                }
            }
            CommentStyle::Line => {
                for line in trimmed.lines() {
                    out.push_str(&rule.comment_prefix);
                    out.push_str(line);
                    out.push_str(eol);
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
        let lines: Vec<&str> = content.lines().collect();

        // Check-only keyword validation rule (when rule defines keywords but no template)
        if rule.keywords.is_some() && rule.template.is_none() {
            let existing_header_range = Self::find_existing_header_range(&lines, rule, config);
            if existing_header_range.is_some() {
                return Ok(FormatResult::Unchanged);
            }
            let kw_list = rule
                .keywords
                .as_ref()
                .map(|k| k.join(", "))
                .unwrap_or_default();
            let diff = format!(
                "Missing required header comment containing: [{}] (rule '{}' is check-only)",
                kw_list, rule.name
            );
            return Ok(FormatResult::Changed {
                new_content: content.to_string(),
                diff,
            });
        }

        let uses_crlf = content.contains("\r\n");
        let eol = if uses_crlf { "\r\n" } else { "\n" };
        let canonical_header = Self::render_header(config, rule, path, content, eol)?;

        // 1. Detect if an existing header is present in the first ~40 lines
        let existing_header_range = Self::find_existing_header_range(&lines, rule, config);

        let new_content = if let Some((start_idx, end_idx)) = existing_header_range {
            // Replace existing header range
            let mut result = String::new();
            for line in lines.iter().take(start_idx) {
                result.push_str(line);
                result.push_str(eol);
            }
            result.push_str(&canonical_header);
            for line in lines.iter().skip(end_idx) {
                result.push_str(line);
                result.push_str(eol);
            }
            if !content.is_empty() && !content.ends_with('\n') && !content.ends_with("\r\n") {
                if uses_crlf && result.ends_with("\r\n") {
                    result.truncate(result.len() - 2);
                } else if result.ends_with('\n') {
                    result.pop();
                }
            }
            result
        } else {
            // Insert new header
            let insert_idx = Self::find_insertion_index(&lines, rule);
            let mut result = String::new();
            for line in lines.iter().take(insert_idx) {
                result.push_str(line);
                result.push_str(eol);
            }
            // If inserting after Go build directives, ensure separating blank line
            if insert_idx > 0
                && lines[insert_idx - 1].trim().starts_with("//")
                && !result.ends_with(&format!("{}{}", eol, eol))
            {
                result.push_str(eol);
            }
            result.push_str(&canonical_header);
            // Add a separating newline if code immediately follows
            if insert_idx < lines.len() && !lines[insert_idx].trim().is_empty() {
                result.push_str(eol);
            }
            for line in lines.iter().skip(insert_idx) {
                result.push_str(line);
                result.push_str(eol);
            }
            if !content.is_empty() && !content.ends_with('\n') && !content.ends_with("\r\n") {
                if uses_crlf && result.ends_with("\r\n") {
                    result.truncate(result.len() - 2);
                } else if result.ends_with('\n') {
                    result.pop();
                }
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

        // Skip shebang if rule preserves it or line-style hash comments
        if (rule.preserve_shebang || rule.comment_prefix.contains('#'))
            && lines[0].starts_with("#!")
        {
            idx += 1;
        }

        // Skip XML prolog if rule preserves it
        if rule.preserve_xml_prolog && idx < lines.len() && lines[idx].starts_with("<?xml") {
            idx += 1;
        }

        // Skip HTML / XML / SVG DOCTYPE declarations (must remain line 1 / above comments)
        if idx < lines.len() && lines[idx].trim().to_lowercase().starts_with("<!doctype") {
            idx += 1;
        }

        // Skip Python / Ruby / editor directives (PEP 263 coding, frozen_string_literal, Emacs -*- modelines)
        while idx < lines.len() && idx <= 3 {
            let lower = lines[idx].trim().to_lowercase();
            if lower.contains("coding=")
                || lower.contains("coding:")
                || lower.contains("frozen_string_literal:")
                || (lower.starts_with('#') && lower.contains("-*-"))
            {
                idx += 1;
            } else {
                break;
            }
        }

        // Skip Go build constraints (`//go:build` and legacy `// +build`)
        while idx < lines.len() {
            let trimmed = lines[idx].trim();
            if trimmed.starts_with("//go:build") || trimmed.starts_with("// +build") {
                idx += 1;
            } else {
                break;
            }
        }

        // If we skipped Go build constraints, skip any blank lines separating them
        if idx > 0 && lines[idx - 1].trim().starts_with("//") {
            while idx < lines.len() && lines[idx].trim().is_empty() {
                idx += 1;
            }
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
        let default_keywords = &config.header.keywords;
        let keywords: &[String] = rule
            .keywords
            .as_deref()
            .unwrap_or(default_keywords.as_slice());
        let contains_keyword = |text: &str| -> bool {
            if keywords.is_empty() {
                true
            } else {
                keywords.iter().any(|k| text.contains(&k.to_lowercase()))
            }
        };

        match rule.style {
            CommentStyle::Block => {
                let start_token = rule.comment_start.as_deref().unwrap_or("/*").trim();
                let end_token = rule.comment_end.as_deref().unwrap_or("*/").trim();

                let mut in_block = false;
                let mut start_idx = None;

                for (i, line) in lines[0..max_search].iter().enumerate() {
                    let trimmed = line.trim();

                    // Allow blank lines, shebangs, xml prologs, doctypes, or build tags before comment
                    if !in_block
                        && (trimmed.is_empty()
                            || trimmed.starts_with("#!")
                            || trimmed.starts_with("<?xml")
                            || trimmed.to_lowercase().starts_with("<!doctype")
                            || trimmed.starts_with("//go:build")
                            || trimmed.starts_with("// +build"))
                    {
                        continue;
                    }

                    if !in_block && trimmed.starts_with(start_token) {
                        in_block = true;
                        start_idx = Some(i);
                    } else if !in_block {
                        // Non-empty, non-comment code statement reached before any header comment
                        break;
                    }

                    if in_block && trimmed.ends_with(end_token) {
                        if let Some(s) = start_idx {
                            let block_text = lines[s..=i].join("\n").to_lowercase();
                            if contains_keyword(&block_text) {
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

                    // Allow blank lines, shebangs, encoding cookies, frozen_string_literal, or modelines before comment
                    if !in_comment
                        && (trimmed.is_empty()
                            || (i == 0 && trimmed.starts_with("#!"))
                            || trimmed.to_lowercase().contains("coding")
                            || trimmed.to_lowercase().contains("frozen_string_literal")
                            || (trimmed.starts_with('#') && trimmed.contains("-*-")))
                    {
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
                            if contains_keyword(&block_text) {
                                return Some((s, i));
                            }
                        }
                        in_comment = false;
                        start_idx = None;
                        // Non-comment code reached after header comment
                        break;
                    } else {
                        // Non-comment code statement reached before any header comment
                        break;
                    }
                }

                if in_comment {
                    if let Some(s) = start_idx {
                        let block_text = lines[s..max_search].join("\n").to_lowercase();
                        if contains_keyword(&block_text) {
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

    #[test]
    fn test_crlf_preservation_and_idempotence() {
        let rules = default_language_rules();
        let java_rule = rules.iter().find(|r| r.name == "c_style_block").unwrap();
        let config = Config::default();

        let initial_crlf = "package com.example;\r\n\r\npublic class CRLF {}\r\n";
        let res = Formatter::format_file(Path::new("CRLF.java"), initial_crlf, java_rule, &config)
            .unwrap();

        let formatted = match res {
            FormatResult::Changed { new_content, .. } => {
                // Assert CRLF is preserved across entire file
                assert!(new_content.contains("\r\n"));
                assert!(!new_content.replace("\r\n", "").contains('\n'));
                assert!(new_content.contains("/*\r\n * Copyright (c)"));
                assert!(new_content.ends_with("\r\n"));
                new_content
            }
            _ => panic!("Expected Changed for unheadered CRLF file"),
        };

        // Verifies idempotence: checking a formatted CRLF file returns Unchanged
        let second_res =
            Formatter::format_file(Path::new("CRLF.java"), &formatted, java_rule, &config).unwrap();
        assert!(matches!(second_res, FormatResult::Unchanged));
    }

    #[test]
    fn test_go_build_tags_preservation() {
        let rules = default_language_rules();
        let go_rule = rules.iter().find(|r| r.name == "c_style_block").unwrap();
        let config = Config::default();

        let initial =
            "//go:build linux && amd64\n// +build linux,amd64\n\npackage main\n\nfunc main() {}\n";
        let res = Formatter::format_file(Path::new("main.go"), initial, go_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                let lines: Vec<&str> = new_content.lines().collect();
                // Build tags must remain at the top
                assert_eq!(lines[0], "//go:build linux && amd64");
                assert_eq!(lines[1], "// +build linux,amd64");
                // Blank line separating build tags from header
                assert_eq!(lines[2], "");
                // Header placed after build tags
                assert_eq!(lines[3], "/*");
                assert!(lines[4].contains("Copyright (c)"));
                assert_eq!(lines[5], " */");
                // Followed by package
                assert!(new_content.contains("package main"));
            }
            _ => panic!("Expected Changed"),
        }
    }

    #[test]
    fn test_python_pep263_encoding_cookie() {
        let rules = default_language_rules();
        let py_rule = rules.iter().find(|r| r.name == "hash_line").unwrap();
        let config = Config::default();

        let initial = "#!/usr/bin/env python3\n# -*- coding: utf-8 -*-\n\nimport os\n";
        let res =
            Formatter::format_file(Path::new("script.py"), initial, py_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                let lines: Vec<&str> = new_content.lines().collect();
                assert_eq!(lines[0], "#!/usr/bin/env python3");
                assert_eq!(lines[1], "# -*- coding: utf-8 -*-");
                assert!(lines[2].starts_with("# Copyright (c)"));
                assert!(new_content.contains("import os"));
            }
            _ => panic!("Expected Changed"),
        }
    }

    #[test]
    fn test_non_header_comment_protected() {
        let rules = default_language_rules();
        let java_rule = rules.iter().find(|r| r.name == "c_style_block").unwrap();
        let config = Config::default();

        // Code begins on line 1, and an internal method has a comment containing "license"
        let initial = "package com.example;\n\npublic class App {\n    // Note: check user license and copyright key\n    public void test() {}\n}\n";
        let res =
            Formatter::format_file(Path::new("App.java"), initial, java_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                // Header is inserted at top before package
                assert!(new_content.starts_with("/*\n * Copyright (c)"));
                // The internal comment inside class App MUST NOT be touched
                assert!(new_content.contains("// Note: check user license and copyright key"));
                assert!(new_content.contains("public void test() {}"));
            }
            _ => panic!("Expected Changed"),
        }
    }

    #[test]
    fn test_empty_file_trailing_newline() {
        let rules = default_language_rules();
        let py_rule = rules.iter().find(|r| r.name == "hash_line").unwrap();
        let config = Config::default();

        let initial = "";
        let res = Formatter::format_file(Path::new("empty.py"), initial, py_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                assert!(new_content.starts_with("# Copyright (c)"));
                assert!(new_content.ends_with('\n'));
            }
            _ => panic!("Expected Changed for empty file"),
        }
    }

    #[test]
    fn test_per_rule_template_override() {
        let mut py_rule = default_language_rules()
            .into_iter()
            .find(|r| r.name == "hash_line")
            .unwrap();
        py_rule.template = Some("Custom Python Header for {{ fs.filename }}".into());

        let config = Config::default();
        let res =
            Formatter::format_file(Path::new("app.py"), "print(1)\n", &py_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                assert!(new_content.starts_with("# Custom Python Header for app.py"));
            }
            _ => panic!("Expected Changed with custom rule template"),
        }
    }

    #[test]
    fn test_custom_command_variable() {
        let py_rule = default_language_rules()
            .into_iter()
            .find(|r| r.name == "hash_line")
            .unwrap();

        let mut config = Config::default();
        config
            .commands
            .insert("greet".into(), "echo Hello from Command".into());
        config.header.template = "Greeting: {{ cmd.greet }}".into();

        let res =
            Formatter::format_file(Path::new("test.py"), "x = 1\n", &py_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                assert!(new_content.contains("# Greeting: Hello from Command"));
            }
            _ => panic!("Expected Changed with custom command variable"),
        }
    }

    #[test]
    fn test_check_only_rule_keywords() {
        let mut check_rule = default_language_rules()
            .into_iter()
            .find(|r| r.name == "hash_line")
            .unwrap();
        check_rule.name = "shell_usage".into();
        check_rule.fixable = false;
        check_rule.keywords = Some(vec!["usage:".into(), "synopsis:".into()]);
        check_rule.template = None;

        let config = Config::default();

        // 1. File with usage header comment passes
        let valid_script =
            "#!/bin/bash\n# Usage: deploy.sh [options]\n# Deploys code\n\necho deploy\n";
        let res_valid =
            Formatter::format_file(Path::new("deploy.sh"), valid_script, &check_rule, &config)
                .unwrap();
        assert_eq!(res_valid, FormatResult::Unchanged);

        // 2. File without usage header comment fails check
        let missing_script = "#!/bin/bash\n\necho deploy\n";
        let res_missing =
            Formatter::format_file(Path::new("deploy.sh"), missing_script, &check_rule, &config)
                .unwrap();
        match res_missing {
            FormatResult::Changed { diff, new_content } => {
                assert!(diff.contains("Missing required header comment"));
                assert!(diff.contains("usage:, synopsis:"));
                assert_eq!(new_content, missing_script);
            }
            _ => panic!("Expected Changed for missing usage header"),
        }
    }

    #[test]
    fn test_html_doctype_preservation() {
        let rules = default_language_rules();
        let xml_rule = rules.iter().find(|r| r.name == "xml_block").unwrap();
        let config = Config::default();

        let initial = "<!DOCTYPE html>\n<html>\n<body>Hello</body>\n</html>\n";
        let res =
            Formatter::format_file(Path::new("index.html"), initial, xml_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                let lines: Vec<&str> = new_content.lines().collect();
                assert_eq!(lines[0], "<!DOCTYPE html>");
                assert_eq!(lines[1], "<!--");
                assert!(lines[2].contains("Copyright (c)"));
                assert_eq!(lines[3], "-->");
                assert!(new_content.contains("<html>"));
            }
            _ => panic!("Expected Changed for html doctype"),
        }
    }

    #[test]
    fn test_ruby_frozen_string_literal_preservation() {
        let rules = default_language_rules();
        let py_rule = rules.iter().find(|r| r.name == "hash_line").unwrap();
        let config = Config::default();

        let initial =
            "#!/usr/bin/env ruby\n# frozen_string_literal: true\n\ndef hello\n  puts 1\nend\n";
        let res =
            Formatter::format_file(Path::new("app.rb"), initial, py_rule, &config).unwrap();

        match res {
            FormatResult::Changed { new_content, .. } => {
                let lines: Vec<&str> = new_content.lines().collect();
                assert_eq!(lines[0], "#!/usr/bin/env ruby");
                assert_eq!(lines[1], "# frozen_string_literal: true");
                assert!(lines[2].starts_with("# Copyright (c)"));
                assert!(new_content.contains("def hello"));
            }
            _ => panic!("Expected Changed for ruby frozen string literal"),
        }
    }
}
