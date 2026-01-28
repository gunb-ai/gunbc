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

## 7. Layer-Owned Mocks + Test Generation

Each layer owns its contract, real SubDAG, mock SubDAG, and test fixtures.
Mocks are faithful to their layer's request/response shape and do not
invent new semantics. This enables test generation that composes layers
and swaps in mocks without changing higher-level tools.

Test strategy:
- Unit tests: format/parse nodes using lower-layer mocks.
- Contract tests: feed layer-specific request fixtures into the mock and
  assert response invariants.
- Composition tests: compose upper layer -> lower mock, verify that
  request/response shapes line up (gistgen -> gist mock is canonical).

This ensures mock behavior is grounded in the real service contract.

---

## 8. Test Generation Matrix (Types, Mocks, Expected Evidence)

The generator should produce tests by pairing each layer with the mock
of the layer directly beneath it. The goal is to show that request/response
shapes line up across boundaries, and that mock behavior is faithful to
the contract.

Test categories (intended to be generated):

1) Layer contract tests (same layer inputs -> same layer mock)
   - Gist layer: CreateRequest fixtures -> Gist mock -> CreateResponse
     Evidence:
     - file names preserved
     - content preserved where not truncated
     - html_url is stable and derived from request
     - required fields present (id, url, files)

2) Layer format/parse tests (layer nodes + lower mock)
   - Gist layer: format_gist_create -> REST mock -> parse_gist_response
     Evidence:
     - REST request contains correct method/path/headers
     - JSON body matches CreateRequest schema
     - parse yields CreateResponse with expected fields

3) Composition tests (upper layer -> lower mock, cardinality-driven)
   The generator should prefer fixtures that vary set cardinality:
   - 0: empty files map (expected reject before REST call)
   - 1: single file (blob path or map path)
   - N>1: multiple files (map path)
   - null: missing/unspecified request (expected reject before REST call)

   These are the "language alignment" cases: they verify the same meaning
   across layers, not just a happy path.

   Evidence:
   - request formation matches intended cardinality
   - invalid cases are rejected at the correct boundary
   - valid cases produce stable mock gist_url and preserved file names

Mocks used:
- Gist mock (faithful CreateResponse)
- REST mock (JSON body)
- HTTP mock (bytes)
- TCP mock (bytes/loopback)

Edge cases the matrix should make evident:
- filename "gistfileNN" rejected or flagged before request formation
- public flag and description are optional but serialized consistently
- empty files map rejected before REST call
- null/missing request rejected before REST call

---

## 9. Op-Set Composition (SetSpec semantics)

When composing DAGs across layers, the op-type universe is the set union
of each layer's op types. This should use `SetSpec<T>` semantics from
`gunbc_ir::algebra`:

- Ops(D): SetSpec<OpTypeName> for a DAG D
- Ops(D_composed) = union of Ops(D_i)
- Empty means no ops (invalid for real DAGs)
- Universal means unknown/any (codegen must refuse or use a fallback)

The union enum used by composed test DAGs must be generated from the
resulting set (deduplicated, deterministic order). It should not be
hand-curated.

---

## 10. Generation Notes

- The request/response structs should be generated from the contract
  types once codegen supports it.
- Avoid expanding into per-field ports; the contract is request-shaped.
  This keeps graphs faithful and compact.

---

## 10. References

- GitHub REST API: Create a gist
  https://docs.github.com/en/rest/gists/gists#create-a-gist
