# uws Skill: Apple iCloud Calendar & Reminders

## Services: `apple-calendar`, `apple-reminders`

Read and manage iCloud Calendar events and Reminders via CalDAV.

## Prerequisites

```bash
uws apple-auth setup   # Step-by-step guide for app-specific password
uws apple-auth status  # Verify authentication
```

Required env vars:
```bash
export UWS_APPLE_ID=your@icloud.com
export UWS_APPLE_APP_PASSWORD=xxxx-xxxx-xxxx-xxxx
```

## Common Commands

```bash
# List all iCloud calendars
uws apple-calendar calendars list

# List events in a calendar (REPORT query)
uws apple-calendar events list \
  --params '{"calendarPath": "/dav/calendars/user/your@icloud.com/home/", "timeRange": "30d"}'

# Create a calendar event (iCalendar format)
uws apple-calendar events create \
  --params '{"calendarPath": "/dav/calendars/user/your@icloud.com/home/"}' \
  --json '{
    "summary": "Doctor appointment",
    "dtstart": "20260310T140000Z",
    "dtend": "20260310T150000Z",
    "description": "Annual checkup"
  }'

# List Reminders
uws apple-reminders lists list

# Create a Reminder
uws apple-reminders tasks create \
  --json '{"summary": "Buy groceries", "due": "20260310"}'
```

## Protocol Notes

- iCloud Calendar uses **CalDAV** (RFC 4791) over HTTPS.
- iCloud Reminders are stored as VTODO components in CalDAV.
- Responses are XML (iCalendar wrapped in WebDAV XML); `uws` returns them as `{"raw": "<xml>"}`.
- Use an **app-specific password** from [appleid.apple.com](https://appleid.apple.com), not your main Apple ID password.

## AI Agent Notes

- CalDAV responses are XML. Parse the `raw` field to extract event data.
- For structured event creation, `uws` accepts a JSON body and converts to iCalendar format internally.
- Always use `--dry-run` before creating or deleting events.
