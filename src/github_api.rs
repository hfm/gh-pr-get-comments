use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::process::Command;

const USER_AGENT: &str = "gh-pr-get-comments";
const API_BASE: &str = "https://api.github.com";
const API_VERSION: &str = "2022-11-28";
const ACCEPT_HEADER: &str = "application/vnd.github+json";
const PER_PAGE: usize = 100;

pub struct GitHubApi {
    client: Client,
    token: String,
    api_base: String,
}

impl GitHubApi {
    pub fn new(hostname: &str) -> anyhow::Result<Self> {
        let token = fetch_token(hostname)?;
        let client = build_client()?;
        let api_base = api_base_for(hostname);
        Ok(Self {
            client,
            token,
            api_base,
        })
    }

    pub fn fetch_pr_comments(
        &self,
        repo: &str,
        pr_number: u64,
        comment_id: Option<u64>,
    ) -> anyhow::Result<Value> {
        if let Some(comment_id) = comment_id {
            return self.fetch_pr_comment(repo, comment_id);
        }

        let comments = self.fetch_all_pr_comments(repo, pr_number)?;
        Ok(Value::Array(comments))
    }

    fn fetch_json<T: DeserializeOwned>(&self, url: &str) -> anyhow::Result<T> {
        let resp = self
            .client
            .get(url)
            .bearer_auth(&self.token)
            .header("Accept", ACCEPT_HEADER)
            .header("X-GitHub-Api-Version", API_VERSION)
            .send()
            .map_err(|e| anyhow::anyhow!("Failed to reach GitHub API: {}", e))?;

        if !resp.status().is_success() {
            let status = resp.status();
            let body = resp.text().unwrap_or_default();
            anyhow::bail!("GitHub API error: {} {}", status, body);
        }

        resp.json::<T>()
            .map_err(|e| anyhow::anyhow!("Failed to parse JSON response: {}", e))
    }

    fn fetch_pr_comment(&self, repo: &str, comment_id: u64) -> anyhow::Result<Value> {
        let url = format!(
            "{}/repos/{}/pulls/comments/{}",
            self.api_base, repo, comment_id
        );
        self.fetch_json(&url)
    }

    fn fetch_all_pr_comments(&self, repo: &str, pr_number: u64) -> anyhow::Result<Vec<Value>> {
        let mut page = 1;
        let mut all = Vec::new();

        loop {
            let url = format!(
                "{}/repos/{}/pulls/{}/comments?per_page={}&page={}",
                self.api_base, repo, pr_number, PER_PAGE, page
            );
            let mut batch: Vec<Value> = self.fetch_json(&url)?;
            let batch_len = batch.len();
            all.append(&mut batch);
            if batch_len < PER_PAGE {
                break;
            }
            page += 1;
        }

        Ok(all)
    }
}

fn api_base_for(hostname: &str) -> String {
    if hostname == "github.com" {
        API_BASE.to_string()
    } else {
        format!("https://{}/api/v3", hostname)
    }
}

fn build_client() -> anyhow::Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to initialize HTTP client: {}", e))
}

fn fetch_token(hostname: &str) -> anyhow::Result<String> {
    if let Some(token) = token_from_env(hostname) {
        return Ok(token);
    }
    if let Some(token) = token_from_gh(hostname) {
        return Ok(token);
    }

    anyhow::bail!("token for {hostname} not found.");
}

fn token_from_env(host: &str) -> Option<String> {
    let keys = if host.eq_ignore_ascii_case("github.com") {
        ["GH_TOKEN", "GITHUB_TOKEN"]
    } else {
        ["GH_ENTERPRISE_TOKEN", "GITHUB_ENTERPRISE_TOKEN"]
    };

    for key in keys {
        if let Ok(token) = std::env::var(key) {
            return Some(token);
        }
    }

    None
}

fn token_from_gh(host: &str) -> Option<String> {
    let output = Command::new("gh")
        .args(["auth", "token", "--secure-storage", "--hostname", host])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }
    let token = String::from_utf8_lossy(&output.stdout).trim().to_string();
    if token.is_empty() { None } else { Some(token) }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn api_base_defaults_to_github() {
        assert_eq!(api_base_for("github.com"), API_BASE);
    }

    #[test]
    fn api_base_for_ghe() {
        assert_eq!(
            api_base_for("ghe.example.com"),
            "https://ghe.example.com/api/v3"
        );
    }
}
