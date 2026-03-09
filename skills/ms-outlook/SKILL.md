# uws Skill: Microsoft Outlook Mail

## Service: `ms-mail`

Read, send, search, and manage Microsoft Outlook email via the Microsoft Graph API.

## Prerequisites

Authenticate first:
```bash
uws ms-auth setup   # First time: register Azure app
uws ms-auth login   # OAuth2 flow
```

## Common Commands

```bash
# List inbox (latest 10)
uws ms-mail messages list --params '{"$top": 10, "$orderby": "receivedDateTime desc"}'

# List unread messages
uws ms-mail messages list --params '{"$filter": "isRead eq false", "$top": 20}'

# Read a specific message
uws ms-mail messages get --params '{"messageId": "MESSAGE_ID"}'

# Send an email
uws ms-mail messages send --json '{
  "message": {
    "subject": "Hello from uws",
    "body": {"contentType": "Text", "content": "Sent via Universal Workspace CLI"},
    "toRecipients": [{"emailAddress": {"address": "recipient@example.com"}}]
  }
}'

# Search messages
uws ms-mail messages list --params '{"$search": "\"project alpha\""}'

# Delete a message
uws ms-mail messages delete --params '{"messageId": "MESSAGE_ID"}'

# Move to folder
uws ms-mail messages move --params '{"messageId": "MESSAGE_ID"}' --json '{"destinationId": "deleteditems"}'

# List mail folders
uws ms-mail mailFolders list
```

## Output Example

```json
{
  "value": [
    {
      "id": "AAMkAGI...",
      "subject": "Q1 Budget Review",
      "from": {"emailAddress": {"name": "Alice", "address": "alice@contoso.com"}},
      "receivedDateTime": "2026-03-08T14:23:00Z",
      "isRead": false,
      "bodyPreview": "Please review the attached budget..."
    }
  ]
}
```

## AI Agent Notes

- Always use `--dry-run` before sending or deleting messages.
- Use `$select` to limit fields returned: `--params '{"$select": "subject,from,receivedDateTime"}'`
- Use `$filter` for server-side filtering to reduce response size.
- The Graph API uses OData query syntax for filtering and ordering.
