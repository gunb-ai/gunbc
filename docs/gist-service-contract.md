# GitHub Gist Layer Contract (Draft)

Status: Draft — January 2026
Purpose: faithful, minimal contract for Create Gist used by gistgen.

---

## 1. Source of Truth

This layer mirrors the GitHub REST API "Create a gist" endpoint:
- Method: POST
- Path: /gists
- Headers: Accept: application/vnd.github+json (recommended), Authorization: Bearer <token>,
  X-GitHub-Api-Version: 2022-11-28
- Body: description (optional), public (bool or string), files (required)
- Auth: fine-grained tokens require "Gists" user permission (write)
- Notes: do not name files with the "gistfileNN" scheme (reserved by GitHub)

This contract is defined to match the service, not convenience.

---

## 2. Type IDs (contract surface)

External boundary:
- External::GitHub::Gist

Request/response types (generated later):
- GitHub::Gist::CreateRequest
- GitHub::Gist::CreateResponse
- GitHub::Gist::Files
- GitHub::Gist::FileInput
- GitHub::Gist::FileMeta

These are request-shaped types, not expanded into many ports.

---

## 3. Data Shapes (logical schema)

GitHub::Gist::CreateRequest
- description: String?
- public: Bool | String
- files: GitHub::Gist::Files

GitHub::Gist::Files
- map<Filename, GitHub::Gist::FileInput>

GitHub::Gist::FileInput
- content: String

GitHub::Gist::CreateResponse (minimal useful subset; faithful to API)
- id: String
- html_url: String
- url: String
- files: map<Filename, GitHub::Gist::FileMeta>
- public: Bool
- description: String?
- truncated: Bool

GitHub::Gist::FileMeta (minimal useful subset; faithful to API)
- filename: String
- type: String?
- language: String?
- raw_url: String
- size: Int
- truncated: Bool
- content: String?
- encoding: String?

Raw REST response should still be available internally for parsing and
future schema expansion, but is not required as a public port.

---

## 4. Gist Layer Ports (Create Gist)

Node: gist_create (opaque or SubDAG; wraps REST layer)

Inputs
- request: GitHub::Gist::CreateRequest
- token: Secret<GithubToken>

Outputs
- response: GitHub::Gist::CreateResponse
- gist_url: String (convenience = response.html_url)

Boundary declaration
- External::GitHub::Gist on gist_url (or on response if preferred)

---

## 5. gistgen Adapters (two entry shapes, one Gist layer)

gistgen constructs the CreateRequest in one of two ways, then calls the
Gist layer:

Path A: single markdown blob
- compose_snapshot -> wrap_as_single_gist_file -> build_gist_create_request
- wrap_as_single_gist_file picks a filename (e.g., "snapshot.md") and
  maps content to { filename: { content } }

Path B: multi-file map
- read_files -> compose_gist_files -> build_gist_create_request

Both paths produce GitHub::Gist::CreateRequest and converge on gist_create.

---

## 6. Mock Behavior (must mirror service shape)

Mock creates a deterministic GitHub::Gist::CreateResponse derived from
the request, including:
- id and html_url (stable, derived from request)
- files map with per-file metadata
- truncated flags set consistently

Mock must accept the same CreateRequest schema as real, and should
preserve file names and content where possible for test inspection.

---

## 7. Generation Notes

- The request/response structs should be generated from the contract
  types once codegen supports it.
- Avoid expanding into per-field ports; the contract is request-shaped.
  This keeps graphs faithful and compact.

---

## 8. References

- GitHub REST API: Create a gist
  https://docs.github.com/en/rest/gists/gists#create-a-gist
