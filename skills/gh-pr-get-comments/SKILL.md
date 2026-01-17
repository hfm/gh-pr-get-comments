---
name: gh-pr-get-comments
description: Fetch GitHub Pull Request inline review comments with `gh pr-get-comments`. Use when you need the comment body from a URL like `https://github.com/:owner/:repo/pull/:pr_number#discussion_rXXXX`.
---

# gh pr-get-comments

## Overview

Use `gh pr-get-comments` to fetch inline PR review comments. Return the command output verbatim.

## Usage

Fetch a review comment by PR comment URL (with `#discussion_r...`).
```bash
gh pr-get-comments --url https://github.com/owner/repo/pull/123#discussion_r456789
```

Fetch all inline review comments for a PR.
```bash
gh pr-get-comments --repo owner/repo --pr 123
```

Fetch a specific review comment by ID.
```bash
gh pr-get-comments --repo owner/repo --pr 123 --comment 456789
gh pr-get-comments --repo owner/repo --comment 456789
```

## Notes

- Prefer `--url` when the user provides a discussion URL.
- Use `--repo` with either `--pr` or `--comment` when a URL is not available.
