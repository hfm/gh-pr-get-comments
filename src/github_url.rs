#[derive(Debug)]
pub struct ParsedUrl {
    pub repo: String,
    pub pr_number: u64,
    pub comment_id: Option<u64>,
}

pub fn parse_github_pr_url(raw: &str) -> anyhow::Result<ParsedUrl> {
    let url = ::url::Url::parse(raw).map_err(|e| anyhow::anyhow!("Failed to parse URL: {}", e))?;

    let invalid = || anyhow::anyhow!("Invalid PR URL format: {}", raw);
    let mut segments = url.path_segments().ok_or_else(invalid)?;
    let (owner, name, pr_segment) = match (
        segments.next(),
        segments.next(),
        segments.next(),
        segments.next(),
    ) {
        (Some(owner), Some(name), Some("pull"), Some(pr_segment)) => (owner, name, pr_segment),
        _ => return Err(invalid()),
    };

    let repo = format!("{}/{}", owner, name);
    let pr_number = pr_segment
        .parse::<u64>()
        .map_err(|_| anyhow::anyhow!("Failed to parse PR number: {}", raw))?;
    let comment_id = match url.fragment() {
        Some(fragment) => Some(parse_comment_fragment(fragment)?),
        None => None,
    };

    Ok(ParsedUrl {
        repo,
        pr_number,
        comment_id,
    })
}

fn parse_comment_fragment(fragment: &str) -> anyhow::Result<u64> {
    match fragment.strip_prefix("discussion_r") {
        Some(rest) => match rest.parse::<u64>() {
            Ok(id) => Ok(id),
            Err(_) => anyhow::bail!("Failed to parse comment ID: #{}", fragment),
        },
        None => anyhow::bail!("Invalid comment ID fragment: #{}", fragment),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_valid_url() {
        let parsed =
            parse_github_pr_url("https://github.com/owner/repo/pull/123#discussion_r456789")
                .unwrap();
        assert_eq!(parsed.repo, "owner/repo");
        assert_eq!(parsed.pr_number, 123);
        assert_eq!(parsed.comment_id, Some(456_789));
    }

    #[test]
    fn accepts_missing_fragment() {
        let parsed = parse_github_pr_url("https://github.com/owner/repo/pull/123").unwrap();
        assert_eq!(parsed.repo, "owner/repo");
        assert_eq!(parsed.pr_number, 123);
        assert_eq!(parsed.comment_id, None);
    }
}
