# Retrospective

Observations and architectural debt noted during development.

---

## `std/patterns.dag` mixes generic patterns with GCP-specific implementations

**Spotted:** 2026-03-09

`dsl/std/patterns.dag` conflates three concerns in a single file:

1. **Generic reusable patterns** — `file_content_matches`, `classify_files`,
   `read_text_files`, `ensure`, `upsert`, `content_upsert`, `credential_chain`,
   `transaction`, `retry`. These are abstract DAG shapes parameterized by
   capabilities. They belong in `std/`.

2. **Concrete auth flows** — `github_oidc`, `metadata_oidc`, `local_auth`.
   These call specific GCP/shell services (`shell.GCloud.AuthPrintAccessToken`,
   `shell.OAuth2.RefreshToken`, `shell.GitHub.OIDCToken`). They are
   GCP/GitHub-specific implementations, not standard library abstractions.

3. **IAM preflight** — `check_iam_binding`, `add_iam_binding`,
   `iam_preflight_check`. These call `gcp.ResourceManager.GetIamPolicy` /
   `SetIamPolicy` directly.

**Why it matters:** `std/` should define provider-agnostic interfaces and
generic patterns. Concrete provider implementations (GCP, GitHub, shell
commands) belong under `extdeps/` or a dedicated `providers/` tree. Having
them in `std/` means every consumer of standard patterns transitively imports
GCP service definitions, and it blurs the line between "what the DSL provides"
and "what a specific cloud integration provides."

**Suggested fix:** Extract auth and IAM functions into provider-specific
modules (e.g., `extdeps/cloud/gcp/auth.dag`, `extdeps/cloud/gcp/iam_preflight.dag`).
If `std/` needs an auth abstraction, define an interface or pattern signature
there and let the provider modules implement it.
