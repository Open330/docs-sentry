use std::process::Command;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RepoMetadata {
    pub name: String,
    pub is_private: bool,
    pub description: String,
}

pub fn list_repositories(org: &str, limit: usize) -> Result<Vec<RepoMetadata>, String> {
    let jq = ".[] | [.name, (.isPrivate|tostring), (.description // \"\")] | @tsv";
    let args = vec![
        "repo".to_string(),
        "list".to_string(),
        org.to_string(),
        "--limit".to_string(),
        limit.to_string(),
        "--json".to_string(),
        "name,isPrivate,description".to_string(),
        "--jq".to_string(),
        jq.to_string(),
    ];

    let result = run_gh(&args)?;
    if !result.success {
        return Err(format!(
            "Failed to list repositories for '{org}': {}",
            best_error(&result)
        ));
    }

    let repos: Vec<RepoMetadata> = result.stdout.lines().filter_map(parse_repo_line).collect();

    if repos.is_empty() {
        return Err(format!(
            "No repositories returned for organization '{org}'. Check access with: gh auth status"
        ));
    }

    Ok(repos)
}

pub fn fetch_readme(org: &str, repo_name: &str) -> Result<Option<String>, String> {
    let endpoint = format!("repos/{org}/{repo_name}/readme");
    let args = vec![
        "api".to_string(),
        "-H".to_string(),
        "Accept: application/vnd.github.raw+json".to_string(),
        endpoint,
    ];

    let result = run_gh(&args)?;
    if result.success {
        return Ok(Some(result.stdout));
    }

    let error_text = best_error(&result);
    if error_text.contains("404") || error_text.contains("Not Found") {
        return Ok(None);
    }

    Err(format!(
        "Unable to fetch README for '{org}/{repo_name}': {error_text}"
    ))
}

#[derive(Debug)]
struct CommandResult {
    success: bool,
    stdout: String,
    stderr: String,
}

fn run_gh(args: &[String]) -> Result<CommandResult, String> {
    let output = Command::new("gh")
        .args(args)
        .output()
        .map_err(|error| format!("Failed to execute gh command: {error}"))?;

    Ok(CommandResult {
        success: output.status.success(),
        stdout: String::from_utf8_lossy(&output.stdout).to_string(),
        stderr: String::from_utf8_lossy(&output.stderr).to_string(),
    })
}

fn best_error(result: &CommandResult) -> String {
    let stderr = result.stderr.trim();
    if !stderr.is_empty() {
        return stderr.to_string();
    }

    let stdout = result.stdout.trim();
    if !stdout.is_empty() {
        return stdout.to_string();
    }

    "unknown gh error".to_string()
}

fn parse_repo_line(line: &str) -> Option<RepoMetadata> {
    let mut parts = line.splitn(3, '\t');
    let name = parts.next()?.trim();
    if name.is_empty() {
        return None;
    }

    let is_private = parts.next().map(str::trim) == Some("true");
    let description = parts.next().unwrap_or("").trim().to_string();

    Some(RepoMetadata {
        name: name.to_string(),
        is_private,
        description,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_repo_line;

    #[test]
    fn parses_repo_tsv_line() {
        let parsed = parse_repo_line("docs-sentry\tfalse\tREADME quality auditor")
            .expect("line should parse");
        assert_eq!(parsed.name, "docs-sentry");
        assert!(!parsed.is_private);
        assert_eq!(parsed.description, "README quality auditor");
    }

    #[test]
    fn ignores_empty_repo_name() {
        assert!(parse_repo_line("\tfalse\tmissing").is_none());
    }
}
