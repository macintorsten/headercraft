/*
 * Copyright (c) 2026 Acme Corp. All rights reserved.
 */

use crate::config::LanguageRule;
use regex::Regex;
use std::path::Path;

pub struct Classifier<'a> {
    rules: &'a [LanguageRule],
}

impl<'a> Classifier<'a> {
    pub fn new(rules: &'a [LanguageRule]) -> Self {
        Self { rules }
    }

    pub fn classify(&self, path: &Path, content: &str) -> Option<&'a LanguageRule> {
        let file_name = path.file_name().and_then(|f| f.to_str()).unwrap_or("");
        let extension = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();

        // 1. Exact filename match
        for rule in self.rules {
            if rule
                .filenames
                .iter()
                .any(|f| f.eq_ignore_ascii_case(file_name))
            {
                return Some(rule);
            }
        }

        // 2. Extension match
        if !extension.is_empty() {
            for rule in self.rules {
                if rule
                    .extensions
                    .iter()
                    .any(|ext| ext.eq_ignore_ascii_case(&extension))
                {
                    return Some(rule);
                }
            }
        }

        // 3. Content pattern / shebang sniffing (check first 10 lines)
        let sample: String = content.lines().take(10).collect::<Vec<_>>().join("\n");
        for rule in self.rules {
            for pattern in &rule.content_patterns {
                if let Ok(re) = Regex::new(pattern) {
                    if re.is_match(&sample) {
                        return Some(rule);
                    }
                }
            }
        }

        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::default_language_rules;

    #[test]
    fn test_extension_classification() {
        let rules = default_language_rules();
        let classifier = Classifier::new(&rules);

        let java = classifier.classify(Path::new("src/Main.java"), "").unwrap();
        assert_eq!(java.name, "c_style_block");

        let py = classifier
            .classify(Path::new("scripts/test.py"), "")
            .unwrap();
        assert_eq!(py.name, "hash_line");

        let xml = classifier.classify(Path::new("pom.xml"), "").unwrap();
        assert_eq!(xml.name, "xml_block");
    }

    #[test]
    fn test_shebang_classification() {
        let rules = default_language_rules();
        let classifier = Classifier::new(&rules);

        let script = classifier
            .classify(Path::new("bin/my-script"), "#!/usr/bin/env bash\necho hi")
            .unwrap();
        assert_eq!(script.name, "hash_line");
    }

    #[test]
    fn test_filename_classification() {
        let rules = default_language_rules();
        let classifier = Classifier::new(&rules);

        let dockerfile = classifier
            .classify(Path::new("Dockerfile"), "FROM ubuntu")
            .unwrap();
        assert_eq!(dockerfile.name, "hash_line");

        let makefile = classifier
            .classify(Path::new("Makefile"), "all:\n\techo")
            .unwrap();
        assert_eq!(makefile.name, "hash_line");
    }

    #[test]
    fn test_case_insensitive_and_unknown() {
        let rules = default_language_rules();
        let classifier = Classifier::new(&rules);

        let java = classifier.classify(Path::new("App.JAVA"), "").unwrap();
        assert_eq!(java.name, "c_style_block");

        let unknown = classifier.classify(Path::new("unknown.xyz123"), "some random data");
        assert!(unknown.is_none());
    }
}
