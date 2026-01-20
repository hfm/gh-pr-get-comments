# gh-pr-get-comments

GitHub CLI extension to fetch inline PR review comments.

## Install

### From GitHub Releases (recommended)
```bash
gh extension install hfm/gh-pr-get-comments
```
On first run, the `gh-pr-get-comments` wrapper downloads the latest release asset into `bin/`.
Release assets must be named `gh-pr-get-comments-<os>-<arch>` (e.g. `gh-pr-get-comments-darwin-arm64`).

### From source (local build)
```bash
cargo build --release

gh extension install .
```
The repository root `gh-pr-get-comments` wrapper runs `target/release/gh-pr-get-comments`, so re-run `cargo build --release` after updates.

## Release

Releases are published by GitHub Actions when a tag matching `vX.Y.Z` is pushed.
```bash
git tag v0.1.0
git push origin v0.1.0
```

## Usage

```bash
# Fetch review comments for a PR
gh pr-get-comments --repo owner/repo --pr 123

# Fetch a single review comment by ID
gh pr-get-comments --repo owner/repo --comment 456789

# Parse repo/pr/comment from a URL
gh pr-get-comments --url https://github.com/owner/repo/pull/123#discussion_r456789

# GitHub Enterprise Server
gh pr-get-comments --hostname ghe.example.com --repo owner/repo --pr 123
```

## Options

- `--repo`: Target repository (owner/repo). Falls back to `GH_REPO` if omitted.
- `--pr`: Pull Request number.
- `--comment`: PR review comment ID.
- `--url`: GitHub PR comment URL.
- `--hostname`: GitHub hostname (for GitHub Enterprise Server). Defaults to github.com.
