# Notion Skill — `uws notion`

Gives AI agents access to Notion: pages, databases, blocks, users, search, and comments.

## Auth
```bash
export NOTION_API_KEY=secret_...
# or
export UWS_NOTION_TOKEN=secret_...
```

Create an Internal Integration at https://www.notion.so/my-integrations and share your workspace with it.

## Quick Reference

| Resource | Method | Description |
|---|---|---|
| `pages` | `get` | Get page by ID |
| `pages` | `create` | Create a page |
| `pages` | `update` | Update page properties |
| `pages` | `archive` | Archive (soft-delete) a page |
| `databases` | `list` | List accessible databases |
| `databases` | `get` | Get database by ID |
| `databases` | `query` | Query database rows |
| `databases` | `create` | Create a database |
| `blocks` | `get` | Get a block |
| `blocks` | `list` | List block children (page content) |
| `blocks` | `append` | Append blocks to a page |
| `blocks` | `delete` | Delete a block |
| `users` | `list` | List workspace users |
| `users` | `me` | Get bot user |
| `search` | `query` | Search everything |
| `comments` | `list` | List comments |
| `comments` | `create` | Add a comment |

## Examples

```bash
# Search for pages
uws notion search query --params '{"query":"Q4 planning","filter":{"object":"page"}}'

# Get a page
uws notion pages get --params '{"page_id":"abc123def456"}'

# Get page content (blocks)
uws notion blocks list --params '{"block_id":"abc123def456"}'

# Query a database
uws notion databases query --params '{"database_id":"db123"}' --json '{
  "filter": {"property": "Status", "select": {"equals": "In Progress"}}
}'

# Create a page in a database
uws notion pages create --json '{
  "parent": {"database_id": "db123"},
  "properties": {
    "Name": {"title": [{"text": {"content": "New Task"}}]},
    "Status": {"select": {"name": "Todo"}}
  }
}' --dry-run

# Append content to a page
uws notion blocks append --params '{"block_id":"page123"}' --json '{
  "children": [{
    "object": "block",
    "type": "paragraph",
    "paragraph": {"rich_text": [{"text": {"content": "New paragraph"}}]}
  }]
}'

# List users
uws notion users list
```

## Agent Rules

1. **Always `--dry-run` before creating or modifying pages.**
2. Use `search query` before creating pages to avoid duplicates.
3. The `Notion-Version` header (`2022-06-28`) is automatically set.
4. Page IDs and block IDs are UUIDs — extract them from URLs or search results.
