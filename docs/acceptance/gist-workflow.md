# Gist Workflow Acceptance Criteria

## Goal
`make gist` should create a real GitHub gist and print the upload URL in output.

## Preconditions
- `GITHUB_TOKEN` is set to a valid GitHub token with `gist` scope.
- Network access to `api.github.com` is available.

## Criteria
1. `make gist-dry` exits `0` and prints:
   - `parse_transport_services_github_gist_github_Gist_Create.url: ...`
2. `make gist` exits `0` and prints:
   - `parse_transport_services_github_gist_github_Gist_Create.url: https://gist.github.com/...`
3. `make gist` fails closed when auth is missing/invalid:
   - non-zero exit
   - `POST /gists failed (status 401)` appears in output.

## Verification Commands
```bash
make gist-dry
GITHUB_TOKEN=... make gist
```
