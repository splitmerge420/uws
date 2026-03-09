# uws Skill: Microsoft Teams

## Service: `ms-teams`

Read and manage Microsoft Teams channels, messages, and meetings via the Microsoft Graph API.

## Common Commands

```bash
# List joined teams
uws ms-teams teams list

# List channels in a team
uws ms-teams teams channels list --params '{"teamId": "TEAM_ID"}'

# List messages in a channel
uws ms-teams teams channels messages list \
  --params '{"teamId": "TEAM_ID", "channelId": "CHANNEL_ID"}'

# Send a message to a channel
uws ms-teams teams channels messages create \
  --params '{"teamId": "TEAM_ID", "channelId": "CHANNEL_ID"}' \
  --json '{"body": {"content": "Hello from uws CLI!"}}'

# List chats
uws ms-teams chats list

# List messages in a chat
uws ms-teams chats messages list --params '{"chatId": "CHAT_ID"}'

# Create a Teams meeting
uws ms-calendar events insert \
  --params '{"calendarId": "primary"}' \
  --json '{
    "subject": "Team sync",
    "isOnlineMeeting": true,
    "onlineMeetingProvider": "teamsForBusiness",
    "start": {"dateTime": "2026-03-10T10:00:00", "timeZone": "UTC"},
    "end": {"dateTime": "2026-03-10T11:00:00", "timeZone": "UTC"}
  }'
```

## AI Agent Notes

- Teams messages support HTML content in `body.contentType: "html"`.
- Use `--dry-run` before sending messages to channels.
- Meeting creation goes through the Calendar API with `isOnlineMeeting: true`.
