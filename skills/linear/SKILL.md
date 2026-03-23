# Linear Skill — `uws linear`

Gives AI agents access to Linear.app: issues, projects, teams, cycles (sprints), labels, workflow states, comments, and organization data.

## Auth
```bash
export LINEAR_API_KEY=lin_api_...
# or
export UWS_LINEAR_TOKEN=lin_api_...
```

## Quick Reference

| Resource | Method | Description |
|---|---|---|
| `issues` | `list` | List issues |
| `issues` | `get` | Get issue by ID |
| `issues` | `create` | Create issue |
| `issues` | `update` | Update issue |
| `issues` | `assign` | Assign to user |
| `issues` | `search` | Search issues |
| `projects` | `list` | List projects |
| `projects` | `create` | Create project |
| `teams` | `list` | List teams |
| `teams` | `members` | List team members |
| `cycles` | `list` | List cycles/sprints |
| `cycles` | `issues` | Issues in a cycle |
| `labels` | `list` | List labels |
| `states` | `list` | List workflow states |
| `comments` | `list` | List comments |
| `comments` | `create` | Add comment |
| `me` | `get` | Authenticated user |
| `org` | `get` | Organization details |

## Examples

```bash
# List open issues for a team
uws linear issues list --params '{"teamId":"TEAM-1","states":["Todo","In Progress"]}'

# Get a specific issue
uws linear issues get --params '{"id":"ENG-123"}'

# Create an issue
uws linear issues create --json '{
  "title": "Fix login timeout bug",
  "description": "Users are being logged out after 5 minutes",
  "teamId": "TEAM-1",
  "priority": 2
}' --dry-run

# Search issues
uws linear issues search --params '{"query":"authentication bug"}'

# List the current cycle for a team
uws linear cycles list --params '{"teamId":"TEAM-1","isActive":"true"}'

# Add a comment
uws linear comments create --json '{
  "issueId": "ENG-123",
  "body": "Fixed in PR #42"
}'

# Get authenticated user
uws linear me get
```

## Agent Rules

1. **Always `--dry-run` before creating or updating issues.**
2. Use `issues search` before creating new issues to avoid duplicates.
3. When assigning issues, verify user IDs with `teams members` first.
4. Use `cycles issues` to get sprint context before prioritizing work.
