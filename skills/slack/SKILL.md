# Slack Skill — `uws slack`

Gives AI agents direct access to Slack workspace data: channels, messages, threads, users, reactions, files, search, DMs, reminders, and bookmarks.

## Auth
```bash
export SLACK_BOT_TOKEN=xoxb-...
# or
export UWS_SLACK_TOKEN=xoxb-...
```

## Quick Reference

| Resource | Method | Description |
|---|---|---|
| `channels` | `list` | List all channels |
| `channels` | `history` | Get message history |
| `channels` | `create` | Create a channel |
| `messages` | `post` | Post a message |
| `messages` | `update` | Update a message |
| `messages` | `delete` | Delete a message |
| `messages` | `schedule` | Schedule a message |
| `threads` | `replies` | Get thread replies |
| `threads` | `reply` | Reply in a thread |
| `users` | `list` | List all users |
| `users` | `info` | Get user info |
| `users` | `presence` | Get user presence |
| `reactions` | `add` | Add emoji reaction |
| `reactions` | `remove` | Remove reaction |
| `files` | `list` | List files |
| `search` | `messages` | Search messages |
| `search` | `all` | Search everything |
| `dm` | `open` | Open a DM |
| `workspace` | `info` | Get workspace info |
| `reminders` | `add` | Create reminder |
| `bookmarks` | `list` | List channel bookmarks |

## Examples

```bash
# List all channels
uws slack channels list

# Get message history for a channel
uws slack channels history --params '{"channel":"C1234567890","limit":"20"}'

# Post a message
uws slack messages post --json '{"channel":"general","text":"Hello team!"}' --dry-run
uws slack messages post --json '{"channel":"general","text":"Hello team!"}'

# Post a message with Block Kit
uws slack messages post --json '{
  "channel": "general",
  "blocks": [{"type":"section","text":{"type":"mrkdwn","text":"*Hello!*"}}]
}'

# Reply in a thread
uws slack threads reply --json '{
  "channel": "C1234567890",
  "thread_ts": "1234567890.123456",
  "text": "Replying in thread"
}'

# Search messages
uws slack search messages --params '{"query":"budget Q4 2026","count":"10"}'

# Set status
uws slack status set --json '{"profile":{"status_text":"In a meeting","status_emoji":":spiral_calendar_pad:"}}'

# Add a reminder
uws slack reminders add --json '{"text":"Review PR #42","time":"1234567890"}'

# Open a DM
uws slack dm open --json '{"users":"U1234567890"}'
```

## Agent Rules

1. **Always `--dry-run` before posting messages, reactions, or reminders.**
2. **Confirm with the user before deleting messages or archiving channels.**
3. Use `channels history` to read context before posting.
4. Use `search messages` to find relevant conversations before creating new ones.
5. Thread replies are preferred over top-level messages to keep channels clean.
