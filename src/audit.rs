use crate::github::RepoMetadata;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditStatus {
    Strong,
    NeedsWork,
    Weak,
    MissingReadme,
    FetchError,
}

impl AuditStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Strong => "strong",
            Self::NeedsWork => "needs-work",
            Self::Weak => "weak",
            Self::MissingReadme => "missing-readme",
            Self::FetchError => "fetch-error",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepoAudit {
    pub repo: RepoMetadata,
    pub score: u8,
    pub status: AuditStatus,
    pub has_readme: bool,
    pub missing_required: Vec<&'static str>,
    pub missing_recommended: Vec<&'static str>,
    pub notes: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AuditSummary {
    pub total: usize,
    pub with_readme: usize,
    pub below_threshold: usize,
    pub average_score: f32,
}

#[derive(Clone, Copy)]
struct WeightedCheck {
    name: &'static str,
    weight: u8,
    required: bool,
    heading_aliases: &'static [&'static str],
    content_patterns: &'static [&'static str],
}

const CHECKS: [WeightedCheck; 10] = [
    WeightedCheck {
        name: "Features",
        weight: 18,
        required: true,
        heading_aliases: &["features", "feature"],
        content_patterns: &[],
    },
    WeightedCheck {
        name: "Quick Start",
        weight: 16,
        required: true,
        heading_aliases: &["quick start", "getting started"],
        content_patterns: &[],
    },
    WeightedCheck {
        name: "Architecture",
        weight: 14,
        required: true,
        heading_aliases: &["architecture"],
        content_patterns: &[],
    },
    WeightedCheck {
        name: "License",
        weight: 14,
        required: true,
        heading_aliases: &["license"],
        content_patterns: &[],
    },
    WeightedCheck {
        name: "Tech Stack",
        weight: 10,
        required: false,
        heading_aliases: &["tech stack"],
        content_patterns: &[],
    },
    WeightedCheck {
        name: "Usage",
        weight: 8,
        required: false,
        heading_aliases: &["usage", "cli options", "commands"],
        content_patterns: &[],
    },
    WeightedCheck {
        name: "Run Tests",
        weight: 8,
        required: false,
        heading_aliases: &["run tests", "testing", "tests"],
        content_patterns: &[],
    },
    WeightedCheck {
        name: "Deploy",
        weight: 6,
        required: false,
        heading_aliases: &["deploy", "deployment"],
        content_patterns: &[],
    },
    WeightedCheck {
        name: "Centered Hero",
        weight: 3,
        required: false,
        heading_aliases: &[],
        content_patterns: &["<p align=\"center\">"],
    },
    WeightedCheck {
        name: "Agent Header",
        weight: 3,
        required: false,
        heading_aliases: &[],
        content_patterns: &["quickstart-for-agents.vercel.app/api/header.svg"],
    },
];

#[derive(Clone, Copy)]
struct FenceSpec {
    marker: char,
    len: usize,
}

pub fn audit_repo(
    repo: &RepoMetadata,
    readme: Option<&str>,
    min_score: u8,
    _strict: bool,
) -> RepoAudit {
    let Some(contents) = readme else {
        return RepoAudit {
            repo: repo.clone(),
            score: 0,
            status: AuditStatus::MissingReadme,
            has_readme: false,
            missing_required: required_section_names(),
            missing_recommended: recommended_section_names(),
            notes: vec!["README file is missing.".to_string()],
        };
    };

    let lowered = contents.to_ascii_lowercase();
    let headings = extract_normalized_headings(&lowered);
    let mut score: u16 = 0;
    let mut missing_required = Vec::new();
    let mut missing_recommended = Vec::new();

    for check in CHECKS {
        let matched = matches_check(&check, &lowered, &headings);

        if matched {
            score += u16::from(check.weight);
        } else if check.required {
            missing_required.push(check.name);
        } else {
            missing_recommended.push(check.name);
        }
    }

    let score = score.min(100) as u8;
    let status = if score >= 85 {
        AuditStatus::Strong
    } else if score >= 60 {
        AuditStatus::NeedsWork
    } else {
        AuditStatus::Weak
    };

    let mut notes = Vec::new();
    if score < min_score {
        notes.push(format!("Below target score ({score} < {min_score})."));
    }

    RepoAudit {
        repo: repo.clone(),
        score,
        status,
        has_readme: true,
        missing_required,
        missing_recommended,
        notes,
    }
}

pub fn audit_fetch_error(repo: &RepoMetadata, error: &str, min_score: u8) -> RepoAudit {
    let mut notes = vec![format!("README fetch failed: {error}")];
    if 0 < min_score {
        notes.push(format!("Below target score (0 < {min_score})."));
    }

    RepoAudit {
        repo: repo.clone(),
        score: 0,
        status: AuditStatus::FetchError,
        has_readme: false,
        missing_required: Vec::new(),
        missing_recommended: Vec::new(),
        notes,
    }
}

pub fn sort_audits(audits: &mut [RepoAudit]) {
    audits.sort_by(|left, right| {
        left.score
            .cmp(&right.score)
            .then_with(|| left.repo.name.cmp(&right.repo.name))
    });
}

pub fn summarize(audits: &[RepoAudit], min_score: u8) -> AuditSummary {
    let total = audits.len();
    if total == 0 {
        return AuditSummary {
            total: 0,
            with_readme: 0,
            below_threshold: 0,
            average_score: 0.0,
        };
    }

    let with_readme = audits.iter().filter(|audit| audit.has_readme).count();
    let below_threshold = audits
        .iter()
        .filter(|audit| audit.score < min_score)
        .count();
    let score_sum: u32 = audits.iter().map(|audit| u32::from(audit.score)).sum();
    let average_score = score_sum as f32 / total as f32;

    AuditSummary {
        total,
        with_readme,
        below_threshold,
        average_score,
    }
}

fn required_section_names() -> Vec<&'static str> {
    CHECKS
        .iter()
        .filter(|check| check.required)
        .map(|check| check.name)
        .collect()
}

fn recommended_section_names() -> Vec<&'static str> {
    CHECKS
        .iter()
        .filter(|check| !check.required)
        .map(|check| check.name)
        .collect()
}

fn matches_check(check: &WeightedCheck, readme_lower: &str, headings: &[String]) -> bool {
    let heading_match = check.heading_aliases.iter().any(|alias| {
        let normalized_alias = normalize_phrase(alias);
        headings
            .iter()
            .any(|heading| contains_token_sequence(heading, &normalized_alias))
    });

    if heading_match {
        return true;
    }

    check
        .content_patterns
        .iter()
        .any(|pattern| readme_lower.contains(&pattern.to_ascii_lowercase()))
}

fn extract_normalized_headings(readme_lower: &str) -> Vec<String> {
    let mut headings = Vec::new();
    let mut fence: Option<FenceSpec> = None;
    let mut front_matter_possible = true;
    let mut front_matter_end_index: Option<usize> = None;
    let lines: Vec<&str> = readme_lower.lines().collect();
    let mut index = 0usize;

    while index < lines.len() {
        let trimmed = lines[index].trim_start();
        let trimmed_both = lines[index].trim();

        if let Some(end_index) = front_matter_end_index
            && index <= end_index
        {
            if index == end_index {
                front_matter_end_index = None;
            }
            index += 1;
            continue;
        }

        if front_matter_possible && trimmed_both.is_empty() {
            index += 1;
            continue;
        }

        if front_matter_possible {
            front_matter_possible = false;
            if (trimmed_both == "---" || trimmed_both == "+++")
                && let Some(end_index) = find_front_matter_end(&lines, index, trimmed_both)
            {
                front_matter_end_index = Some(end_index);
                index += 1;
                continue;
            }
        }

        if let Some(current_fence) = fence {
            if is_closing_fence(trimmed, current_fence) {
                fence = None;
            }
            index += 1;
            continue;
        }

        if let Some(opening_fence) = parse_opening_fence(trimmed) {
            fence = Some(opening_fence);
            index += 1;
            continue;
        }

        if trimmed.starts_with('#') {
            let heading = trimmed.trim_start_matches('#').trim();
            if !heading.is_empty() {
                let normalized = normalize_phrase(heading);
                if !normalized.is_empty() {
                    headings.push(normalized);
                }
            }
            index += 1;
            continue;
        }

        if index + 1 < lines.len() {
            let heading_candidate = lines[index].trim();
            let underline_candidate = lines[index + 1].trim();

            if !heading_candidate.is_empty() && is_setext_underline(underline_candidate) {
                let normalized = normalize_phrase(heading_candidate);
                if !normalized.is_empty() {
                    headings.push(normalized);
                }
                index += 2;
                continue;
            }
        }

        index += 1;
    }

    headings
}

fn find_front_matter_end(lines: &[&str], opening_index: usize, delimiter: &str) -> Option<usize> {
    let mut has_metadata_like_line = false;

    for (index, line) in lines.iter().enumerate().skip(opening_index + 1) {
        let trimmed = line.trim();

        let is_closing_delimiter = trimmed == delimiter || (delimiter == "---" && trimmed == "...");
        if is_closing_delimiter {
            return has_metadata_like_line.then_some(index);
        }

        let looks_like_yaml_metadata = trimmed.contains(':');
        let looks_like_toml_metadata = delimiter == "+++" && trimmed.contains('=');
        if !trimmed.is_empty() && (looks_like_yaml_metadata || looks_like_toml_metadata) {
            has_metadata_like_line = true;
        }
    }

    None
}

fn parse_opening_fence(trimmed_line: &str) -> Option<FenceSpec> {
    let mut chars = trimmed_line.chars();
    let marker = chars.next()?;
    if marker != '`' && marker != '~' {
        return None;
    }

    let len = trimmed_line
        .chars()
        .take_while(|character| *character == marker)
        .count();
    if len < 3 {
        return None;
    }

    Some(FenceSpec { marker, len })
}

fn is_closing_fence(trimmed_line: &str, opening_fence: FenceSpec) -> bool {
    let marker_count = trimmed_line
        .chars()
        .take_while(|character| *character == opening_fence.marker)
        .count();
    if marker_count < opening_fence.len {
        return false;
    }

    let trailing = trimmed_line.chars().skip(marker_count).collect::<String>();
    trailing.trim().is_empty()
}

fn is_setext_underline(trimmed_line: &str) -> bool {
    if trimmed_line.len() < 3 {
        return false;
    }

    let mut chars = trimmed_line.chars();
    let marker = chars.next().unwrap_or_default();
    if marker != '=' && marker != '-' {
        return false;
    }

    chars.all(|character| character == marker)
}

fn contains_token_sequence(heading: &str, alias: &str) -> bool {
    let heading_tokens: Vec<&str> = heading.split_whitespace().collect();
    let alias_tokens: Vec<&str> = alias.split_whitespace().collect();

    if alias_tokens.is_empty() || heading_tokens.len() < alias_tokens.len() {
        return false;
    }

    heading_tokens
        .windows(alias_tokens.len())
        .any(|window| window == alias_tokens)
}

fn normalize_phrase(value: &str) -> String {
    let mut normalized = String::new();
    let mut last_space = false;

    for character in value.chars() {
        if character.is_ascii_alphanumeric() {
            normalized.push(character.to_ascii_lowercase());
            last_space = false;
        } else if !last_space {
            normalized.push(' ');
            last_space = true;
        }
    }

    normalized
        .split_whitespace()
        .collect::<Vec<&str>>()
        .join(" ")
}

#[cfg(test)]
mod tests {
    use super::{AuditStatus, audit_fetch_error, audit_repo, summarize};
    use crate::github::RepoMetadata;

    fn example_repo() -> RepoMetadata {
        RepoMetadata {
            name: "example".to_string(),
            is_private: false,
            description: "Example repo".to_string(),
        }
    }

    #[test]
    fn scores_template_like_readme_high() {
        let readme = "
<p align=\"center\"></p>
## Features
## Quick Start
## Architecture
## Tech Stack
## Usage
### Run Tests
## Deploy
## License
quickstart-for-agents.vercel.app/api/header.svg
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert_eq!(audit.score, 100);
        assert_eq!(audit.status, AuditStatus::Strong);
        assert!(audit.missing_required.is_empty());
    }

    #[test]
    fn marks_missing_readme() {
        let audit = audit_repo(&example_repo(), None, 70, false);
        assert_eq!(audit.score, 0);
        assert_eq!(audit.status, AuditStatus::MissingReadme);
        assert!(!audit.missing_required.is_empty());
    }

    #[test]
    fn marks_fetch_error_separately() {
        let audit = audit_fetch_error(&example_repo(), "HTTP 502", 70);
        assert_eq!(audit.score, 0);
        assert_eq!(audit.status, AuditStatus::FetchError);
        assert!(audit.missing_required.is_empty());
    }

    #[test]
    fn summary_counts_thresholds() {
        let readme = "## Features\n## Quick Start\n## Architecture\n## License\n";
        let passing = audit_repo(&example_repo(), Some(readme), 50, false);
        let failing = audit_repo(
            &RepoMetadata {
                name: "no-readme".to_string(),
                is_private: false,
                description: "".to_string(),
            },
            None,
            50,
            false,
        );

        let summary = summarize(&[passing, failing], 50);
        assert_eq!(summary.total, 2);
        assert_eq!(summary.with_readme, 1);
        assert_eq!(summary.below_threshold, 1);
    }

    #[test]
    fn accepts_heading_variants_without_exact_hash_pattern() {
        let readme = "
# Product
### Feature Overview
## Getting Started Guide
#### Architecture Overview
## License and Legal
## Testing Strategy
### Deployment Notes
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert!(audit.missing_required.is_empty());
        assert!(audit.score >= 62);
    }

    #[test]
    fn strict_flag_only_affects_exit_behavior_not_scoring() {
        let readme = "## Features\n## Quick Start\n## Architecture\n## License\n";
        let regular = audit_repo(&example_repo(), Some(readme), 70, false);
        let strict = audit_repo(&example_repo(), Some(readme), 70, true);

        assert_eq!(regular.score, strict.score);
        assert_eq!(regular.missing_required, strict.missing_required);
    }

    #[test]
    fn does_not_match_alias_inside_other_words() {
        let readme = "
## Features
## Quick Start
## Architecture
## License
## Contests
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert!(audit.missing_recommended.contains(&"Run Tests"));
    }

    #[test]
    fn ignores_headings_inside_fenced_code_blocks() {
        let readme = "
```markdown
## Features
```
## Quick Start
## Architecture
## License
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert!(audit.missing_required.contains(&"Features"));
    }

    #[test]
    fn supports_setext_style_headings() {
        let readme = "
Features
--------
Quick Start
===========
Architecture
------------
License
-------
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert!(audit.missing_required.is_empty());
    }

    #[test]
    fn does_not_close_fence_with_different_marker() {
        let readme = "
```markdown
## Features
~~~
## Quick Start
```
## Architecture
## License
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert!(audit.missing_required.contains(&"Features"));
        assert!(audit.missing_required.contains(&"Quick Start"));
    }

    #[test]
    fn does_not_close_fence_with_shorter_length() {
        let readme = "
````markdown
## Features
```
## Quick Start
````
## Architecture
## License
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert!(audit.missing_required.contains(&"Features"));
        assert!(audit.missing_required.contains(&"Quick Start"));
    }

    #[test]
    fn ignores_yaml_front_matter_for_heading_detection() {
        let readme = "
---
title: Example project
features: false
---
## Quick Start
## Architecture
## License
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert!(audit.missing_required.contains(&"Features"));
    }

    #[test]
    fn ignores_toml_front_matter_for_heading_detection() {
        let readme = "
+++
title = \"Example project\"
features = false
# Features
+++
## Quick Start
## Architecture
## License
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert!(audit.missing_required.contains(&"Features"));
    }

    #[test]
    fn does_not_treat_unclosed_horizontal_rule_as_front_matter() {
        let readme = "
---
## Features
## Quick Start
## Architecture
## License
";

        let audit = audit_repo(&example_repo(), Some(readme), 70, false);
        assert!(audit.missing_required.is_empty());
    }
}
