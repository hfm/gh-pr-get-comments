mod github_api;
mod github_url;
use anyhow::Context;
use clap::Parser;
use github_api::GitHubApi;
use github_url::parse_github_pr_url;
use serde_json::Value;

#[derive(Parser, Debug)]
#[command(
    name = "gh-pr-get-comments",
    override_usage = "gh pr-get-comments [OPTIONS]\n       gh-pr-get-comments [OPTIONS]",
    about = "Fetch inline PR comments via GitHub API",
    arg_required_else_help = true,
    group(clap::ArgGroup::new("target").required(true).multiple(true).args(["url", "pr", "comment"])),
    after_help = r#"Examples:
  gh pr-get-comments --repo owner/repo --pr 123
  gh pr-get-comments --repo owner/repo --comment 456789
  gh pr-get-comments --url https://github.com/owner/repo/pull/123#discussion_r456789"#
)]
struct Cli {
    #[arg(long, value_name = "HOST", default_value = "github.com")]
    hostname: String,
    #[arg(short, long, value_name = "OWNER/REPO")]
    repo: Option<String>,
    #[arg(short, long, value_name = "NUMBER")]
    pr: Option<u64>,
    #[arg(short, long, value_name = "ID")]
    comment: Option<u64>,
    #[arg(short, long, value_name = "URL", conflicts_with_all = ["repo", "pr", "comment"])]
    url: Option<String>,
}

fn main() -> anyhow::Result<()> {
    let args = Cli::parse();

    let (hostname, repo, pr_number, comment_id) = if let Some(url) = args.url.as_deref() {
        let parsed = parse_github_pr_url(url)?;
        (
            parsed.hostname,
            parsed.repo,
            parsed.pr_number,
            parsed.comment_id,
        )
    } else {
        (
            args.hostname,
            args.repo
                .or_else(|| std::env::var("GH_REPO").ok())
                .unwrap_or_default(),
            args.pr.unwrap_or(0),
            args.comment,
        )
    };

    validate_repo(&repo)?;

    let hostname = normalize_hostname(&hostname)?;
    let api = GitHubApi::new(hostname.as_str())?;
    let json = api.fetch_pr_comments(&repo, pr_number, comment_id)?;
    print_comments(&json)?;
    Ok(())
}

fn print_comments(json: &Value) -> anyhow::Result<()> {
    if let Some(comments) = json.as_array() {
        comments
            .iter()
            .enumerate()
            .try_for_each(|(index, comment)| {
                if index > 0 {
                    println!();
                }
                print_comment(comment)
            })?;
        return Ok(());
    }

    if json.is_object() {
        return print_comment(json);
    }

    anyhow::bail!("Unexpected GitHub API response format")
}

fn print_comment(comment: &Value) -> anyhow::Result<()> {
    let url = comment
        .get("html_url")
        .and_then(Value::as_str)
        .context("Missing html_url in comment")?;
    let body = comment
        .get("body")
        .and_then(Value::as_str)
        .unwrap_or_default();
    println!("{url}\n\n{body}");
    Ok(())
}

fn validate_repo(repo: &str) -> anyhow::Result<()> {
    match repo.split_once('/') {
        Some((owner, name))
            if !owner.is_empty()
                && !owner.chars().any(char::is_whitespace)
                && !name.is_empty()
                && !name.chars().any(char::is_whitespace)
                && !name.contains('/') =>
        {
            Ok(())
        }
        _ => anyhow::bail!("Specify --repo owner/repo (e.g. --repo owner/repo)"),
    }
}

fn normalize_hostname(raw: &str) -> anyhow::Result<String> {
    if let Ok(url) = url::Url::parse(raw) {
        if let Some(host) = url.host_str() {
            return Ok(host.to_ascii_lowercase());
        }
        anyhow::bail!("Invalid --hostname: {}", raw);
    }

    let host = raw.split(['/', '?', '#']).next().unwrap_or(raw);
    if host.is_empty() {
        anyhow::bail!("Invalid --hostname: {}", raw);
    }
    Ok(host.to_ascii_lowercase())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validates_repo() {
        assert!(validate_repo("owner/repo").is_ok());
        assert!(validate_repo("owner/repo/extra").is_err());
        assert!(validate_repo("owner /repo").is_err());
        assert!(validate_repo("owner/").is_err());
        assert!(validate_repo("/repo").is_err());
        assert!(validate_repo("").is_err());
    }

    #[test]
    fn normalizes_hostname() {
        assert_eq!(normalize_hostname("github.com").unwrap(), "github.com");
        assert_eq!(
            normalize_hostname("https://ghe.example.com/").unwrap(),
            "ghe.example.com"
        );
        assert_eq!(
            normalize_hostname("ghe.example.com/api/v3").unwrap(),
            "ghe.example.com"
        );
    }
}
