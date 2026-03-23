# Figma Skill — `uws figma`

Gives AI agents access to Figma: files, nodes, components, comments, projects, teams, styles, and webhooks.

## Auth
```bash
export FIGMA_TOKEN=figd_...
# or
export UWS_FIGMA_TOKEN=figd_...
```

Get a Personal Access Token from https://www.figma.com/developers/api#access-tokens.

## Quick Reference

| Resource | Method | Description |
|---|---|---|
| `files` | `get` | Get a file |
| `files` | `nodes` | Get specific nodes |
| `files` | `export` | Export nodes as images |
| `files` | `versions` | Get version history |
| `files` | `styles` | List styles in file |
| `components` | `list` | List all components |
| `components` | `get` | Get component by key |
| `components` | `sets` | List component sets |
| `comments` | `list` | List all comments |
| `comments` | `post` | Post a comment |
| `comments` | `delete` | Delete a comment |
| `projects` | `list` | List team projects |
| `projects` | `files` | Files in a project |
| `teams` | `components` | Team library components |
| `teams` | `styles` | Team library styles |
| `me` | `get` | Authenticated user |
| `webhooks` | `list` | List webhooks |
| `webhooks` | `create` | Create webhook |

## Examples

```bash
# Get a file's complete data
uws figma files get --params '{"file_key":"Abc123XYZ"}'

# Export specific nodes as SVG
uws figma files export --params '{"file_key":"Abc123XYZ","ids":"1:2,1:3","format":"svg","scale":"2"}'

# List all components in a file
uws figma components list --params '{"file_key":"Abc123XYZ"}'

# List and read comments
uws figma comments list --params '{"file_key":"Abc123XYZ"}'

# Post a comment (dry-run first)
uws figma comments post --params '{"file_key":"Abc123XYZ"}' --json '{
  "message": "This button color needs to match the design system",
  "client_meta": {"node_id": "1:2"}
}' --dry-run

# List projects in a team
uws figma projects list --params '{"team_id":"TEAM123"}'

# Get team library components
uws figma teams components --params '{"team_id":"TEAM123"}'

# Get authenticated user
uws figma me get
```

## Agent Rules

1. **Always `--dry-run` before posting comments or creating webhooks.**
2. File keys are the alphanumeric string in the Figma URL: `figma.com/file/{FILE_KEY}/...`
3. Use `files export` with `format=svg` for vector-accurate component exports.
4. Component keys differ from node IDs — use `components list` to get the correct key.
