use crate::github::RepoMetadata;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AuditStatus {
    Strong,
    NeedsWork,
    Weak,
    MissingReadme,
}

impl AuditStatus {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Strong => "strong",
            Self::NeedsWork => "needs-work",
            Self::Weak => "weak",
            Self::MissingReadme => "missing-readme",
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
    patterns: &'static [&'static str],
}

const CHECKS: [WeightedCheck; 10] = [
    WeightedCheck {
        name: "Features",
        weight: 18,
        required: true,
        patterns: &["## features"],
    },
    WeightedCheck {
        name: "Quick Start",
        weight: 16,
        required: true,
        patterns: &["## quick start", "## getting started"],
    },
    WeightedCheck {
        name: "Architecture",
        weight: 14,
        required: true,
        patterns: &["## architecture"],
    },
    WeightedCheck {
        name: "License",
        weight: 14,
        required: true,
        patterns: &["## license"],
    },
    WeightedCheck {
        name: "Tech Stack",
        weight: 10,
        required: false,
        patterns: &["## tech stack"],
    },
    WeightedCheck {
        name: "Usage",
        weight: 8,
        required: false,
        patterns: &["## usage", "## cli options", "## commands"],
    },
    WeightedCheck {
        name: "Run Tests",
        weight: 8,
        required: false,
        patterns: &["### run tests", "## testing"],
    },
    WeightedCheck {
        name: "Deploy",
        weight: 6,
        required: false,
        patterns: &["## deploy", "## deployment"],
    },
    WeightedCheck {
        name: "Centered Hero",
        weight: 3,
        required: false,
        patterns: &["<p align=\"center\">"],
    },
    WeightedCheck {
        name: "Agent Header",
        weight: 3,
        required: false,
        patterns: &["quickstart-for-agents.vercel.app/api/header.svg"],
    },
];

pub fn audit_repo(
    repo: &RepoMetadata,
    readme: Option<&str>,
    min_score: u8,
    strict: bool,
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
    let mut score: u16 = 0;
    let mut missing_required = Vec::new();
    let mut missing_recommended = Vec::new();

    for check in CHECKS {
        let matched = check
            .patterns
            .iter()
            .any(|pattern| lowered.contains(&pattern.to_ascii_lowercase()));

        if matched {
            score += u16::from(check.weight);
        } else if check.required {
            missing_required.push(check.name);
        } else {
            missing_recommended.push(check.name);
        }
    }

    if strict {
        let penalty = (missing_required.len() as u16) * 5;
        score = score.saturating_sub(penalty);
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
    if strict && !missing_required.is_empty() {
        notes.push("Strict mode penalty applied due to missing required sections.".to_string());
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

#[cfg(test)]
mod tests {
    use super::{AuditStatus, audit_repo, summarize};
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
}
