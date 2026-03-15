# Invariant Retrospectives

This file is owned by the `gunbc_recent_invariants` worker. It scans the
last few days of `gunbc` `origin/main` commits and identifies recurring
invariants that the codebase appears to be rediscovering.

## Managed Summary

<!-- openclaw:recent-summary:start -->
- Last reviewed at: 2026-03-15T15:41:16-04:00
- Last reviewed head: `c77734be`
- Rolling window: 3 days
- Commits reviewed last run: 40
- Candidate invariants surfaced last run: 2
- Last chosen theme: Downstream stages must consume the authoritative typed or resolved output of the previous stage and must not re-declare or re-derive equivalent IR, type environments, or registries locally.
- Last change target: daglang-lower REST middleware response-classification boundary in derive_middleware_config
- Last code paths changed: 2
<!-- openclaw:recent-summary:end -->

## Managed Latest Retrospective

<!-- openclaw:recent-latest:start -->
### 2026-03-15T15:41:16-04:00

- Head ref: `origin/main` @ `c77734be`
- Rolling window: 3 days
- Commits reviewed: 40
- Candidate invariants surfaced: 2
- Chosen theme: Downstream stages must consume the authoritative typed or resolved output of the previous stage and must not re-declare or re-derive equivalent IR, type environments, or registries locally.
- Change target: daglang-lower REST middleware response-classification boundary in derive_middleware_config
- Code paths changed: 2

#### Candidate Invariants

- Open language and compiler sets must be modeled structurally, not via string tags, duplicated keyword tables, or hardcoded method/type-name dispatch.
- Downstream stages must consume the authoritative typed or resolved output of the previous stage and must not re-declare or re-derive equivalent IR, type environments, or registries locally.

#### Code Paths Changed

- `src/v1/05_graph/daglang-lower/src/lib.rs`
- `src/v1/05_graph/daglang-lower/src/tests.rs`

#### Retrospective-Driven Run

Short retrospective assessment: This was a high-confidence recurrence of the recent “single authority over heuristic re-derivation” theme. `response_provider` had already been made authoritative in lowered service metadata, but REST middleware still had a separate name-based derivation path and could silently omit classification metadata.

Recurring themes used:
CANDIDATE-INVARIANT: Open language and compiler sets must be modeled structurally, not via string tags, duplicated keyword tables, or hardcoded method/type-name dispatch.
CANDIDATE-INVARIANT: Downstream stages must consume the authoritative typed or resolved output of the previous stage and must not re-declare or re-derive equivalent IR, type environments, or registries locally.

Concrete issue selected: in `src/v1/05_graph/daglang-lower/src/lib.rs`, `derive_middleware_config` re-inferred the provider from `service.name` instead of consuming the authoritative service-config path, and it returned no middleware unless `rate_limit` or `retry` existed. That left a real seam where `config { response_provider: GitHub, error_shape: ... }` could stamp correct node metadata but still lose REST response-classification middleware.

Fix and verification: I added `response_provider_for_service()` and reused it for both service metadata and REST middleware in `src/v1/05_graph/daglang-lower/src/lib.rs`, threaded the `Result` boundary through REST spec lowering at `src/v1/05_graph/daglang-lower/src/lib.rs`, and added a regression test in `src/v1/05_graph/daglang-lower/src/tests.rs`. Verified with `cargo test -p daglang-lower`, `cargo clippy --workspace --exclude gunbc-codegen -- -D warnings && cargo clippy -p gunbc-codegen --lib -- -D warnings`, and `cargo test --workspace --exclude gunbc-codegen`.
<!-- openclaw:recent-latest:end -->

## Managed History

<!-- openclaw:recent-history:start -->
- 2026-03-15T15:41:16-04:00 reviewed `c77734be`; commits=40; candidates=2; changed=2; Short retrospective assessment: This was a high-confidence recurrence of the recent “single authority over heuristic...
- 2026-03-15T15:12:17-04:00 reviewed `c77734be`; commits=40; candidates=7; **Assessment**
- 2026-03-15T15:08:11-04:00 reviewed `c77734be`; commits=40; candidates=7; **Assessment**
<!-- openclaw:recent-history:end -->
