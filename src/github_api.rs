use reqwest::blocking::Client;
use serde::de::DeserializeOwned;
use serde_json::Value;
use std::env;
use std::process::Command;

const USER_AGENT: &str = "gh-pr-get-comments";
const API_BASE: &str = "https://api.github.com";
const API_VERSION: &str = "2022-11-28";
const ACCEPT_HEADER: &str = "application/vnd.github+json";
const PER_PAGE: usize = 100;

pub struct GitHubApi {
    client: Client,
    token: String,
}

impl GitHubApi {
    pub fn new() -> anyhow::Result<Self> {
        let token = fetch_token()?;
        let client = build_client()?;
        Ok(Self { client, token })
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
        let url = format!("{}/repos/{}/pulls/comments/{}", API_BASE, repo, comment_id);
        self.fetch_json(&url)
    }

    fn fetch_all_pr_comments(&self, repo: &str, pr_number: u64) -> anyhow::Result<Vec<Value>> {
        let mut page = 1;
        let mut all = Vec::new();

        loop {
            let url = format!(
                "{}/repos/{}/pulls/{}/comments?per_page={}&page={}",
                API_BASE, repo, pr_number, PER_PAGE, page
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

fn build_client() -> anyhow::Result<Client> {
    Client::builder()
        .user_agent(USER_AGENT)
        .build()
        .map_err(|e| anyhow::anyhow!("Failed to initialize HTTP client: {}", e))
}

fn fetch_token() -> anyhow::Result<String> {
    let env_token = env::var("GH_TOKEN")
        .or_else(|_| env::var("GITHUB_TOKEN"))
        .unwrap_or_default()
        .trim()
        .to_string();
    if !env_token.is_empty() {
        return Ok(env_token);
    }

    let output = Command::new("gh")
        .args(["auth", "token"])
        .output()
        .map_err(anyhow::Error::new)?;
    if !output.status.success() {
        anyhow::bail!("{}", String::from_utf8_lossy(&output.stderr));
    }

    Ok(String::from_utf8_lossy(&output.stdout).trim().to_string())
}
