mod github_api;
mod github_url;
use clap::Parser;
use github_api::GitHubApi;
use github_url::parse_github_pr_url;

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

    let (repo, pr_number, comment_id) = if let Some(url) = args.url.as_deref() {
        let parsed = parse_github_pr_url(url)?;
        (parsed.repo, parsed.pr_number, parsed.comment_id)
    } else {
        (
            args.repo
                .or_else(|| std::env::var("GH_REPO").ok())
                .unwrap_or_default(),
            args.pr.unwrap_or(0),
            args.comment,
        )
    };

    validate_repo(&repo)?;

    let api = GitHubApi::new()?;
    let json = api.fetch_pr_comments(&repo, pr_number, comment_id)?;
    println!("{}", json);
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
}
