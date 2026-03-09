# uws Skill: Microsoft OneDrive

## Service: `ms-onedrive`

List, upload, download, and manage files on Microsoft OneDrive via the Microsoft Graph API.

## Common Commands

```bash
# List root files
uws ms-onedrive drive root children list

# List files in a folder
uws ms-onedrive drive items children list --params '{"itemId": "FOLDER_ID"}'

# Get file metadata
uws ms-onedrive drive items get --params '{"itemId": "FILE_ID"}'

# Search OneDrive
uws ms-onedrive drive root search --params '{"q": "budget 2026"}'

# Upload a file
uws ms-onedrive drive items upload --params '{"itemId": "root:/Documents/report.pdf:"}'  --upload /local/path/report.pdf

# Download a file
uws ms-onedrive drive items content get --params '{"itemId": "FILE_ID"}' --output /local/path/downloaded.pdf

# Create a folder
uws ms-onedrive drive items children create \
  --params '{"itemId": "root"}' \
  --json '{"name": "New Folder", "folder": {}, "@microsoft.graph.conflictBehavior": "rename"}'

# Delete a file
uws ms-onedrive drive items delete --params '{"itemId": "FILE_ID"}'

# Share a file (create sharing link)
uws ms-onedrive drive items createLink \
  --params '{"itemId": "FILE_ID"}' \
  --json '{"type": "view", "scope": "anonymous"}'
```

## AI Agent Notes

- Use `--dry-run` before any destructive operations (delete, overwrite).
- File paths can be referenced as `root:/path/to/file.txt:` in `itemId`.
- The Graph API supports delta queries for efficient sync: use `drive/root/delta`.
