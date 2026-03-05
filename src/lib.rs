pub mod audit;
pub mod config;
pub mod github;
pub mod output;

use audit::{audit_fetch_error, audit_repo, sort_audits, summarize};
use config::{Config, OutputFormat};
use github::{fetch_readme, list_repositories};
use output::{render_json, render_markdown, render_table};

#[derive(Debug)]
pub struct RunResult {
    pub output: String,
    pub below_threshold: usize,
}

pub fn run(config: &Config) -> Result<RunResult, String> {
    let repos = list_repositories(&config.org, config.limit)?;
    let repos: Vec<_> = repos
        .into_iter()
        .filter(|repo| config.include_private || !repo.is_private)
        .collect();

    if repos.is_empty() {
        return Err(
            "No repositories remain after filtering. Try --include-private or increase --limit."
                .to_string(),
        );
    }

    let mut audits = Vec::with_capacity(repos.len());
    for repo in repos {
        match fetch_readme(&config.org, &repo.name) {
            Ok(readme) => audits.push(audit_repo(
                &repo,
                readme.as_deref(),
                config.min_score,
                config.strict,
            )),
            Err(error) => audits.push(audit_fetch_error(&repo, &error, config.min_score)),
        }
    }

    sort_audits(&mut audits);
    let summary = summarize(&audits, config.min_score);

    let output = match config.format {
        OutputFormat::Table => render_table(&audits, &config.org, config.min_score),
        OutputFormat::Markdown => render_markdown(&audits, &config.org, config.min_score),
        OutputFormat::Json => render_json(&audits, &config.org, config.min_score),
    };

    Ok(RunResult {
        output,
        below_threshold: summary.below_threshold,
    })
}
