use std::fmt::Write;

use crate::audit::{RepoAudit, summarize};

pub fn render_table(audits: &[RepoAudit], org: &str, min_score: u8) -> String {
    let summary = summarize(audits, min_score);
    let mut output = String::new();

    let _ = writeln!(output, "Organization: {org}");
    let _ = writeln!(output, "Repositories scanned: {}", summary.total);
    let _ = writeln!(output, "READMEs found: {}", summary.with_readme);
    let _ = writeln!(output, "Average score: {:.1}", summary.average_score);
    let _ = writeln!(
        output,
        "Below target ({}): {}",
        min_score, summary.below_threshold
    );
    let _ = writeln!(output);

    let rows = build_rows(audits);
    if rows.is_empty() {
        return output;
    }

    let repo_width = max_width("Repo", rows.iter().map(|row| row.repo.as_str()));
    let score_width = max_width("Score", rows.iter().map(|row| row.score.as_str()));
    let status_width = max_width("Status", rows.iter().map(|row| row.status.as_str()));
    let required_width = max_width(
        "Missing Required",
        rows.iter().map(|row| row.missing_required.as_str()),
    );

    let _ = writeln!(
        output,
        "{:<repo_width$}  {:>score_width$}  {:<status_width$}  {:<required_width$}  Notes",
        "Repo", "Score", "Status", "Missing Required",
    );
    let _ = writeln!(
        output,
        "{}  {}  {}  {}  -----",
        "-".repeat(repo_width),
        "-".repeat(score_width),
        "-".repeat(status_width),
        "-".repeat(required_width),
    );

    for row in rows {
        let _ = writeln!(
            output,
            "{:<repo_width$}  {:>score_width$}  {:<status_width$}  {:<required_width$}  {}",
            row.repo, row.score, row.status, row.missing_required, row.notes,
        );
    }

    output
}

pub fn render_markdown(audits: &[RepoAudit], org: &str, min_score: u8) -> String {
    let summary = summarize(audits, min_score);
    let mut output = String::new();

    let _ = writeln!(output, "## Documentation Audit Report");
    let _ = writeln!(output);
    let _ = writeln!(output, "- Organization: `{org}`");
    let _ = writeln!(output, "- Repositories scanned: {}", summary.total);
    let _ = writeln!(output, "- READMEs found: {}", summary.with_readme);
    let _ = writeln!(output, "- Average score: {:.1}", summary.average_score);
    let _ = writeln!(
        output,
        "- Below target (`{min_score}`): {}",
        summary.below_threshold
    );
    let _ = writeln!(output);
    let _ = writeln!(
        output,
        "| Repo | Score | Status | Missing Required | Missing Recommended | Notes |"
    );
    let _ = writeln!(output, "| --- | ---: | --- | --- | --- | --- |");

    for audit in audits {
        let _ = writeln!(
            output,
            "| `{}` | {} | {} | {} | {} | {} |",
            audit.repo.name,
            audit.score,
            audit.status.as_str(),
            format_missing(&audit.missing_required),
            format_missing(&audit.missing_recommended),
            format_notes(&audit.notes),
        );
    }

    output
}

pub fn render_json(audits: &[RepoAudit], org: &str, min_score: u8) -> String {
    let summary = summarize(audits, min_score);
    let mut output = String::new();

    let _ = writeln!(output, "{{");
    let _ = writeln!(output, "  \"organization\": \"{}\",", escape_json(org));
    let _ = writeln!(output, "  \"repositories_scanned\": {},", summary.total);
    let _ = writeln!(output, "  \"readmes_found\": {},", summary.with_readme);
    let _ = writeln!(output, "  \"average_score\": {:.2},", summary.average_score);
    let _ = writeln!(output, "  \"below_target\": {},", summary.below_threshold);
    let _ = writeln!(output, "  \"repos\": [");

    for (index, audit) in audits.iter().enumerate() {
        let comma = if index + 1 == audits.len() { "" } else { "," };
        let _ = writeln!(output, "    {{");
        let _ = writeln!(
            output,
            "      \"name\": \"{}\",",
            escape_json(&audit.repo.name)
        );
        let _ = writeln!(output, "      \"score\": {},", audit.score);
        let _ = writeln!(
            output,
            "      \"status\": \"{}\",",
            escape_json(audit.status.as_str())
        );
        let _ = writeln!(
            output,
            "      \"missing_required\": {},",
            render_json_string_list(&audit.missing_required)
        );
        let _ = writeln!(
            output,
            "      \"missing_recommended\": {},",
            render_json_string_list(&audit.missing_recommended)
        );
        let _ = writeln!(
            output,
            "      \"notes\": {}",
            render_json_note_list(&audit.notes)
        );
        let _ = writeln!(output, "    }}{comma}");
    }

    let _ = writeln!(output, "  ]");
    let _ = writeln!(output, "}}");

    output
}

#[derive(Debug)]
struct Row {
    repo: String,
    score: String,
    status: String,
    missing_required: String,
    notes: String,
}

fn build_rows(audits: &[RepoAudit]) -> Vec<Row> {
    audits
        .iter()
        .map(|audit| Row {
            repo: audit.repo.name.clone(),
            score: audit.score.to_string(),
            status: audit.status.as_str().to_string(),
            missing_required: format_missing(&audit.missing_required),
            notes: format_notes(&audit.notes),
        })
        .collect()
}

fn max_width<'a>(header: &'a str, values: impl Iterator<Item = &'a str>) -> usize {
    values
        .map(str::len)
        .fold(header.len(), |current_max, next| current_max.max(next))
}

fn format_missing(items: &[&str]) -> String {
    if items.is_empty() {
        "-".to_string()
    } else {
        items.join(", ")
    }
}

fn format_notes(notes: &[String]) -> String {
    if notes.is_empty() {
        "-".to_string()
    } else {
        notes.join(" | ")
    }
}

fn render_json_string_list(items: &[&str]) -> String {
    let values: Vec<String> = items
        .iter()
        .map(|item| format!("\"{}\"", escape_json(item)))
        .collect();
    format!("[{}]", values.join(", "))
}

fn render_json_note_list(items: &[String]) -> String {
    let values: Vec<String> = items
        .iter()
        .map(|item| format!("\"{}\"", escape_json(item)))
        .collect();
    format!("[{}]", values.join(", "))
}

fn escape_json(value: &str) -> String {
    let mut escaped = String::new();

    for character in value.chars() {
        match character {
            '"' => escaped.push_str("\\\""),
            '\\' => escaped.push_str("\\\\"),
            '\n' => escaped.push_str("\\n"),
            '\r' => escaped.push_str("\\r"),
            '\t' => escaped.push_str("\\t"),
            c if c.is_control() => {
                let _ = write!(escaped, "\\u{:04x}", c as u32);
            }
            _ => escaped.push(character),
        }
    }

    escaped
}

#[cfg(test)]
mod tests {
    use super::{render_markdown, render_table};
    use crate::audit::{AuditStatus, RepoAudit};
    use crate::github::RepoMetadata;

    #[test]
    fn table_contains_headers() {
        let reports = sample_reports();
        let output = render_table(&reports, "Open330", 70);
        assert!(output.contains("Organization: Open330"));
        assert!(output.contains("Missing Required"));
        assert!(output.contains("docs-sentry"));
    }

    #[test]
    fn markdown_contains_table() {
        let reports = sample_reports();
        let output = render_markdown(&reports, "Open330", 70);
        assert!(output.contains("## Documentation Audit Report"));
        assert!(output.contains("| Repo | Score | Status |"));
        assert!(output.contains("`docs-sentry`"));
    }

    fn sample_reports() -> Vec<RepoAudit> {
        vec![RepoAudit {
            repo: RepoMetadata {
                name: "docs-sentry".to_string(),
                is_private: false,
                description: "README audit tool".to_string(),
            },
            score: 96,
            status: AuditStatus::Strong,
            has_readme: true,
            missing_required: Vec::new(),
            missing_recommended: vec!["Deploy"],
            notes: Vec::new(),
        }]
    }
}
