#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum OutputFormat {
    Table,
    Markdown,
    Json,
}

impl OutputFormat {
    pub fn parse(value: &str) -> Result<Self, String> {
        match value {
            "table" => Ok(Self::Table),
            "markdown" => Ok(Self::Markdown),
            "json" => Ok(Self::Json),
            _ => Err(format!(
                "Unsupported format '{value}'. Expected one of: table, markdown, json."
            )),
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Table => "table",
            Self::Markdown => "markdown",
            Self::Json => "json",
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Config {
    pub org: String,
    pub limit: usize,
    pub min_score: u8,
    pub format: OutputFormat,
    pub include_private: bool,
    pub strict: bool,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            org: "Open330".to_string(),
            limit: 100,
            min_score: 70,
            format: OutputFormat::Table,
            include_private: false,
            strict: false,
        }
    }
}

impl Config {
    pub fn parse(args: Vec<String>) -> Result<Self, String> {
        let mut config = Self::default();
        let mut index = 0usize;

        while index < args.len() {
            match args[index].as_str() {
                "--org" => {
                    config.org = take_value(&args, index, "--org")?;
                    if config.org.trim().is_empty() {
                        return Err("--org cannot be empty.".to_string());
                    }
                    index += 2;
                }
                "--limit" => {
                    let raw = take_value(&args, index, "--limit")?;
                    config.limit = raw.parse::<usize>().map_err(|_| {
                        format!("Invalid --limit value '{raw}'. Expected a positive integer.")
                    })?;
                    if config.limit == 0 {
                        return Err("--limit must be greater than 0.".to_string());
                    }
                    index += 2;
                }
                "--min-score" => {
                    let raw = take_value(&args, index, "--min-score")?;
                    config.min_score = raw.parse::<u8>().map_err(|_| {
                        format!("Invalid --min-score value '{raw}'. Expected 0-100.")
                    })?;
                    if config.min_score > 100 {
                        return Err("--min-score must be in range 0-100.".to_string());
                    }
                    index += 2;
                }
                "--format" => {
                    let raw = take_value(&args, index, "--format")?;
                    config.format = OutputFormat::parse(&raw)?;
                    index += 2;
                }
                "--include-private" => {
                    config.include_private = true;
                    index += 1;
                }
                "--strict" => {
                    config.strict = true;
                    index += 1;
                }
                unknown => {
                    return Err(format!("Unknown option '{unknown}'.\n\n{}", Self::usage()));
                }
            }
        }

        Ok(config)
    }

    pub fn usage() -> &'static str {
        "docs-sentry - Audit GitHub repository README quality\n\nUSAGE:\n  docs-sentry [OPTIONS]\n\nOPTIONS:\n  --org <ORG>              GitHub organization to scan (default: Open330)\n  --limit <N>              Max repositories to fetch (default: 100)\n  --min-score <0-100>      Minimum acceptable score (default: 70)\n  --format <FORMAT>        Output format: table, markdown, json (default: table)\n  --include-private         Include private repositories in the report\n  --strict                  Exit with code 2 when repos are below --min-score\n  -h, --help               Print help\n\nEXAMPLES:\n  docs-sentry --org Open330 --format markdown\n  docs-sentry --org Open330 --limit 50 --min-score 80 --strict\n  docs-sentry --org rust-lang --format json\n"
    }
}

fn take_value(args: &[String], index: usize, flag: &str) -> Result<String, String> {
    args.get(index + 1)
        .cloned()
        .ok_or_else(|| format!("Missing value for {flag}."))
}

#[cfg(test)]
mod tests {
    use super::{Config, OutputFormat};

    #[test]
    fn parses_defaults() {
        let config = Config::parse(Vec::new()).expect("default parse should work");
        assert_eq!(config.org, "Open330");
        assert_eq!(config.limit, 100);
        assert_eq!(config.min_score, 70);
        assert_eq!(config.format, OutputFormat::Table);
        assert!(!config.include_private);
        assert!(!config.strict);
    }

    #[test]
    fn parses_custom_flags() {
        let config = Config::parse(vec![
            "--org".to_string(),
            "ExampleOrg".to_string(),
            "--limit".to_string(),
            "25".to_string(),
            "--min-score".to_string(),
            "88".to_string(),
            "--format".to_string(),
            "markdown".to_string(),
            "--include-private".to_string(),
            "--strict".to_string(),
        ])
        .expect("custom parse should work");

        assert_eq!(config.org, "ExampleOrg");
        assert_eq!(config.limit, 25);
        assert_eq!(config.min_score, 88);
        assert_eq!(config.format, OutputFormat::Markdown);
        assert!(config.include_private);
        assert!(config.strict);
    }
}
