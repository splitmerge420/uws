# uws Skill: Apple iCloud Contacts

## Service: `apple-contacts`

Read and manage iCloud Contacts via CardDAV.

## Prerequisites

Same as `apple-calendar` — requires `UWS_APPLE_ID` and `UWS_APPLE_APP_PASSWORD`.

## Common Commands

```bash
# List all address books
uws apple-contacts addressbooks list

# List all contacts (vCards)
uws apple-contacts contacts list \
  --params '{"addressBookPath": "/dav/addressbooks/user/your@icloud.com/card/"}'

# Get a specific contact
uws apple-contacts contacts get \
  --params '{"contactPath": "/dav/addressbooks/user/your@icloud.com/card/CONTACT_UID.vcf"}'

# Create a contact
uws apple-contacts contacts create \
  --json '{
    "fn": "Jane Doe",
    "email": "jane@example.com",
    "tel": "+1-555-0100",
    "org": "Acme Corp"
  }'

# Delete a contact
uws apple-contacts contacts delete \
  --params '{"contactPath": "/dav/addressbooks/user/your@icloud.com/card/CONTACT_UID.vcf"}'
```

## Protocol Notes

- iCloud Contacts uses **CardDAV** (RFC 6352) over HTTPS.
- Contacts are stored as vCard 3.0/4.0 format.
- Responses are XML with embedded vCard data; `uws` returns `{"raw": "<xml>"}`.

## AI Agent Notes

- Parse the `raw` field to extract vCard data from CardDAV responses.
- Use `--dry-run` before any create/delete operations.
- Contact UIDs are UUIDs; generate a new UUID for each new contact.
