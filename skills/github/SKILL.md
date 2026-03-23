---
name: github
version: 1.0.0
description: "GitHub REST API: repos, issues, PRs, releases, Actions, search, and more."
metadata:
  openclaw:
    category: "developer"
    requires:
      bins: ["uws"]
      env: ["GITHUB_TOKEN"]
    cliHelp: "uws github --help"
---

# github — Universal Workspace CLI

> **PREREQUISITE:** Set `GITHUB_TOKEN` or `UWS_GITHUB_TOKEN` to a GitHub personal access token (PAT). In GitHub Actions, `GITHUB_TOKEN` is available automatically.

```bash
uws github <resource> <method> [--params '<JSON>'] [--json '<JSON>'] [--dry-run]
```

## Resources

| Resource | Methods |
|---|---|
| `repos` | `list`, `list-org`, `list-user`, `get`, `create`, `create-org`, `delete` |
| `issues` | `list`, `list-assigned`, `get`, `create`, `update`, `comments`, `comment` |
| `pulls` | `list`, `get`, `create`, `merge`, `reviews`, `files` |
| `releases` | `list`, `get`, `latest`, `create` |
| `actions` | `list`, `runs`, `jobs`, `dispatch` |
| `search` | `repos`, `issues`, `code`, `users`, `commits` |
| `user` | `me` |
| `users` | `get`, `repos` |
| `notifications` | `list`, `mark-read` |
| `stars` | `list`, `list-user`, `star`, `unstar` |
| `gists` | `list`, `get`, `create` |
| `contents` | `get`, `create` |
| `commits` | `list`, `get` |
| `orgs` | `list`, `get`, `members` |
| `labels` | `list`, `create` |
| `milestones` | `list` |

## Authentication

```bash
export GITHUB_TOKEN=ghp_xxxxxxxxxxxxxxxxxxxx

# Or use the gh CLI extension (auto-injects token):
gh extension install splitmerge420/uws
gh uws github repos list
```

## Common Commands

### Repositories

```bash
# List your repos
uws github repos list

# Get a specific repo
uws github repos get --params '{"owner":"octocat","repo":"Hello-World"}'

# Create a repo
uws github repos create --json '{"name":"my-repo","private":false,"description":"..."}'

# List org repos
uws github repos list-org --params '{"org":"my-org","type":"all"}'
```

### Issues

```bash
# List open issues
uws github issues list \
  --params '{"owner":"octocat","repo":"Hello-World","state":"open","per_page":25}'

# Get a specific issue
uws github issues get \
  --params '{"owner":"octocat","repo":"Hello-World","issue_number":42}'

# Create an issue (dry-run first!)
uws github issues create \
  --params '{"owner":"octocat","repo":"Hello-World"}' \
  --json '{"title":"Found a bug","body":"Steps to reproduce...","labels":["bug"]}' \
  --dry-run

uws github issues create \
  --params '{"owner":"octocat","repo":"Hello-World"}' \
  --json '{"title":"Found a bug","body":"Steps to reproduce...","labels":["bug"]}'

# Close an issue
uws github issues update \
  --params '{"owner":"octocat","repo":"Hello-World","issue_number":42}' \
  --json '{"state":"closed"}' --dry-run
```

### Pull Requests

```bash
# List open PRs
uws github pulls list \
  --params '{"owner":"octocat","repo":"Hello-World","state":"open"}'

# Get a specific PR
uws github pulls get \
  --params '{"owner":"octocat","repo":"Hello-World","pull_number":1}'

# List files changed in a PR
uws github pulls files \
  --params '{"owner":"octocat","repo":"Hello-World","pull_number":1}'

# List reviews
uws github pulls reviews \
  --params '{"owner":"octocat","repo":"Hello-World","pull_number":1}'
```

### Releases

```bash
# List releases
uws github releases list --params '{"owner":"octocat","repo":"Hello-World"}'

# Get latest release
uws github releases latest --params '{"owner":"octocat","repo":"Hello-World"}'

# Create a release (dry-run first!)
uws github releases create \
  --params '{"owner":"octocat","repo":"Hello-World"}' \
  --json '{"tag_name":"v1.0.0","name":"v1.0.0","body":"### What'"'"'s new\n- Feature A\n- Bug fix"}' \
  --dry-run
```

### GitHub Actions

```bash
# List workflows
uws github actions list --params '{"owner":"octocat","repo":"Hello-World"}'

# List recent workflow runs
uws github actions runs \
  --params '{"owner":"octocat","repo":"Hello-World","status":"failure","per_page":5}'

# List jobs for a run
uws github actions jobs \
  --params '{"owner":"octocat","repo":"Hello-World","run_id":"12345678"}'

# Trigger a workflow dispatch
uws github actions dispatch \
  --params '{"owner":"octocat","repo":"Hello-World","workflow_id":"ci.yml"}' \
  --json '{"ref":"main","inputs":{"environment":"staging"}}' \
  --dry-run
```

### Search

```bash
# Search repos
uws github search repos --params '{"q":"language:rust stars:>1000","sort":"stars","order":"desc"}'

# Search issues and PRs
uws github search issues \
  --params '{"q":"is:open is:issue label:bug repo:octocat/Hello-World"}'

# Search code
uws github search code \
  --params '{"q":"ProviderDriver repo:splitmerge420/uws"}'

# Search users
uws github search users --params '{"q":"location:seattle followers:>100"}'
```

### User & Notifications

```bash
# Get authenticated user
uws github user me

# Get a specific user
uws github users get --params '{"username":"octocat"}'

# List notifications
uws github notifications list --params '{"all":false,"per_page":20}'

# Mark all notifications as read
uws github notifications mark-read --dry-run
```

### Stars

```bash
# List your starred repos
uws github stars list --params '{"per_page":30}'

# Star a repo
uws github stars star --params '{"owner":"splitmerge420","repo":"uws"}' --dry-run

# List what a user has starred
uws github stars list-user --params '{"username":"octocat"}'
```

### File Contents

```bash
# Get file contents (base64-encoded, decode with jq)
uws github contents get \
  --params '{"owner":"octocat","repo":"Hello-World","path":"README.md"}' \
  | jq -r '.content | @base64d'

# Create or update a file (dry-run first!)
uws github contents create \
  --params '{"owner":"octocat","repo":"Hello-World","path":"notes/todo.md"}' \
  --json '{"message":"Add todo","content":"IyBUT0RPCi0gWyBdIFRhc2sgMQo=","branch":"main"}' \
  --dry-run
```

### Commits

```bash
# List recent commits
uws github commits list \
  --params '{"owner":"octocat","repo":"Hello-World","per_page":10}'

# Get a specific commit
uws github commits get \
  --params '{"owner":"octocat","repo":"Hello-World","ref":"abc1234"}'
```

### Organizations

```bash
# List your organizations
uws github orgs list

# List org members
uws github orgs members --params '{"org":"my-org"}'
```

## Output format

All output is JSON. Use `jq` to extract fields:

```bash
# Extract just the repo names
uws github repos list | jq '[.[].full_name]'

# Get open issue titles and numbers
uws github issues list \
  --params '{"owner":"octocat","repo":"Hello-World","state":"open"}' \
  | jq '[.[] | {number: .number, title: .title}]'

# Check if CI is passing
uws github actions runs \
  --params '{"owner":"octocat","repo":"Hello-World","per_page":1}' \
  | jq '.workflow_runs[0].conclusion'
```

## AI Agent Notes

- **Always use `--dry-run` before write operations** (create, update, merge, delete, dispatch)
- **Never execute `repos delete` without explicit user confirmation**
- **Path params go in `--params`**: `owner`, `repo`, `issue_number`, `pull_number`, `run_id`, etc.
- **Query params also go in `--params`**: `state`, `per_page`, `q`, `sort`, `order`
- **Request body goes in `--json`**: `title`, `body`, `labels`, `tag_name`, `ref`, etc.
- Token is auto-injected in GitHub Actions; for local use, set `GITHUB_TOKEN` in the environment
- If a command fails with `GitHub token not found`, ensure `GITHUB_TOKEN` is set

## GitHub Actions integration

```yaml
- name: List recent failing CI runs
  uses: splitmerge420/uws@main
  with:
    command: github actions runs --params '{"owner":"${{ github.repository_owner }}","repo":"${{ github.event.repository.name }}","status":"failure","per_page":5}'
  id: failing-runs

- name: Parse results
  run: echo '${{ steps.failing-runs.outputs.result }}' | jq '.workflow_runs[].name'
```
