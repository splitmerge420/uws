# uws GitHub Skill

## Overview

This skill enables GitHub Copilot (and any other agent) to use `uws` to read and
write GitHub resources — Issues, Pull Requests, Actions runs, Releases, Code Search,
GitHub Models, and Notifications — through the same universal command grammar as all
other `uws` providers.

**Key insight:** `GITHUB_TOKEN` is automatically injected in every GitHub Actions
workflow, making `uws github` a zero-config provider for all CI/CD use cases.

---

## Authentication

```bash
# In GitHub Actions — no setup needed, GITHUB_TOKEN is auto-injected
# Locally
export GITHUB_TOKEN="ghp_your_personal_access_token"

# Via gh CLI (automatically forwards token)
gh extension install splitmerge420/uws
gh uws github issues list --params '{"owner":"acme","repo":"api"}'
```

Required scopes depend on the operation:
- Read issues/PRs/releases: `repo:read` (or public_repo for public repos)
- Create/update issues/PRs: `repo`
- Actions: `actions:read` / `actions:write`
- GitHub Models: `models:read`
- Notifications: `notifications`

---

## Command Reference

### Issues

```bash
# List open issues
uws github-issues list --params '{"owner":"acme","repo":"api","state":"open"}'

# Get a specific issue
uws github-issues get --params '{"owner":"acme","repo":"api","issue_number":42}'

# Create an issue
uws github-issues create --json '{
  "owner": "acme",
  "repo": "api",
  "title": "Fix auth timeout",
  "body": "The OAuth2 flow times out after 30s on slow connections.",
  "labels": ["bug", "auth"]
}'

# Close an issue
uws github-issues update --json '{"owner":"acme","repo":"api","issue_number":42,"state":"closed"}'

# List issues assigned to me
uws github-issues list --params '{"assignee":"@me","state":"open"}'
```

### Pull Requests

```bash
# List open PRs
uws github-pulls list --params '{"owner":"acme","repo":"api","state":"open"}'

# Get PR diff
uws github-pulls get --params '{"owner":"acme","repo":"api","pull_number":7}'

# List files changed in a PR
uws github-pulls files --params '{"owner":"acme","repo":"api","pull_number":7}'

# Merge a PR (requires repo write)
uws github-pulls merge --json '{"owner":"acme","repo":"api","pull_number":7,"merge_method":"squash"}'
```

### GitHub Actions

```bash
# List recent workflow runs
uws github-actions runs list --params '{"owner":"acme","repo":"api","per_page":10}'

# Get a specific run
uws github-actions runs get --params '{"owner":"acme","repo":"api","run_id":123456789}'

# Trigger a workflow
uws github-actions dispatches create --json '{
  "owner": "acme",
  "repo": "api",
  "workflow_id": "deploy.yml",
  "ref": "main",
  "inputs": {"environment": "staging"}
}'

# List failed jobs in a run
uws github-actions jobs list --params '{"owner":"acme","repo":"api","run_id":123456789}'
```

### Releases

```bash
# List releases
uws github-releases list --params '{"owner":"acme","repo":"api"}'

# Get the latest release
uws github-releases latest --params '{"owner":"acme","repo":"api"}'

# Create a release
uws github-releases create --json '{
  "owner": "acme",
  "repo": "api",
  "tag_name": "v2.1.0",
  "name": "v2.1.0 — Performance improvements",
  "body": "See CHANGELOG.md for details.",
  "draft": false,
  "prerelease": false
}'
```

### Code Search

```bash
# Search for a function across all repos
uws github-search code --params '{"q":"JanusRouter language:rust"}'

# Search within a specific repo
uws github-search code --params '{"q":"TODO repo:acme/api"}'

# Search for secrets accidentally committed (audit)
uws github-search code --params '{"q":"AKIA org:acme"}' --dry-run
```

### GitHub Models (AI inference)

```bash
# List available models
uws github-models list

# Run inference with a specific model
uws github-models chat --json '{
  "model": "gpt-4o",
  "messages": [{"role":"user","content":"Explain async Rust in one paragraph"}]
}'

# Use the cheapest free model
uws github-models chat --json '{
  "model": "meta-llama-3.1-70b-instruct",
  "messages": [{"role":"user","content":"Write a bash one-liner to list large files"}]
}'
```

**Note:** GitHub Models uses the same OpenAI-compatible API format as `uws ai claude`.
When `GITHUB_TOKEN` is set and `ANTHROPIC_API_KEY` is not, `uws ai` will automatically
route to GitHub Models as a free fallback.

### Notifications

```bash
# List unread notifications
uws github-notifications list --params '{"all":false}'

# Mark all as read
uws github-notifications mark-read --json '{"last_read_at":"2026-03-22T00:00:00Z"}'
```

---

## Usage in GitHub Actions

```yaml
# .github/workflows/workspace-report.yml
# Sends a weekly email digest of open issues + unread Gmail
name: Weekly Workspace Report

on:
  schedule:
    - cron: '0 9 * * 1'  # Every Monday at 9am UTC
  workflow_dispatch:

jobs:
  report:
    runs-on: ubuntu-latest
    steps:
      - name: Install uws
        uses: splitmerge420/uws@v1

      - name: Get open issues
        uses: splitmerge420/uws@v1
        id: issues
        with:
          command: "github-issues list --params '{\"owner\":\"${{ github.repository_owner }}\",\"repo\":\"${{ github.event.repository.name }}\",\"state\":\"open\"}'"
          github-token: ${{ secrets.GITHUB_TOKEN }}

      - name: Post summary
        run: echo "${{ steps.issues.outputs.result }}" | python3 -c "
          import json, sys
          issues = json.load(sys.stdin)
          print(f'## Open Issues: {len(issues)}')
          for i in issues[:5]:
              print(f'- #{i[\"number\"]}: {i[\"title\"]}')
          " >> \$GITHUB_STEP_SUMMARY
```

---

## Agent Decision Guide

| Goal | Command |
|---|---|
| Find a GitHub issue by description | `uws github-search code --params '{"q":"<description> type:issue"}'` |
| List recent CI failures | `uws github-actions runs list --params '{"status":"failure","per_page":5}'` |
| Create a PR comment | `uws github-issues comments create` |
| Check if a release exists | `uws github-releases latest --params '{"owner":"...","repo":"..."}'` |
| Run AI inference for free | `uws github-models chat --json '{"model":"gpt-4o-mini","messages":[...]}'` |
| Get my unread notifications | `uws github-notifications list` |

---

## Integration with Janus v2 Council

`uws github-models` plugs directly into the Janus v2 multi-model council as a
cost-free inference backend. When all paid API keys are absent, Janus routes Tier-1
queries through GitHub Models automatically.

```python
# toolchain/janus_runner.py picks this up automatically when:
#   - GITHUB_TOKEN is set
#   - no other API keys are present
# The GitHub Models model used is configured in janus/janus_config.yaml:
#   copilot:
#     role: enterprise
#     model: gpt-4o
#     endpoint: "https://api.githubcopilot.com/chat/completions"
#     env_var: GITHUB_TOKEN
```

---

## Compatible Agents

- GitHub Copilot (native — GITHUB_TOKEN auto-injected)
- Claude (via ANTHROPIC_API_KEY + `uws` tool)
- Gemini (via GEMINI_API_KEY + `uws` tool)
- Manus (via skills/github/SKILL.md installation)
- Any MCP-compatible agent (via `python mcp_server/server.py`)
