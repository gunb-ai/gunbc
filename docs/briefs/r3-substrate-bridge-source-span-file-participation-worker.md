# Worker brief — Substrate bridge: `SourceSpan.file` participation checks (1 of 2)

**Sub-issue**: gunbc#1958 (parented under #1939 Substrate Mgr lane).
**Sibling**: gunbc#1959 closed 2026-05-07 — already retired by PR #1272.
**Authority anchors**: `docs/briefs/bridge-retirement-audit-sourcespan-family.md` (19-row enumeration); `docs/r3-program-plan.md` **§5 Y4 scope-clarification** (current line 353 anchor — verify section heading at HEAD); `src/v3/std/bridge_ledger.dag:125-129` row `bridge_source_span_file_participation_retired` (status=`Open`); `docs/r3-structure.md:115` umbrella-gate framing.

## Ledger-discipline preamble (P2 single-authority)

**The umbrella ledger row stays `Open`.** Per `docs/r3-structure.md:115` and Director acceptance #1130 / dispatch #1139 (2026-04-29): partial string-check retirement was **explicitly rejected** because parallel participation rules would remain. The umbrella's green predicate is *"no production code path consults `SourceSpan.file` for participation/inclusion logic"* — all-or-nothing. Production inclusion sites enumerated at `r3-structure.md:115` (in `lens_apply.rs` / `lower.rs` / `emit.rs`) are NOT in this Substrate slice's scope; they retire under their owners.

This PR's outcome = **retire the bootstrap.rs subset of audit-packet rows #2 + #6 only**. Receipt = update the audit-packet enumeration table at `docs/briefs/bridge-retirement-audit-sourcespan-family.md` to mark rows #2 + #6 retired with the PR # citation. **Do NOT mutate `bridge_ledger.dag`** for this PR — the umbrella row stays `Open` until ALL production sites in the audit packet (rows #1, #3-19 minus test-only / out-of-family) are retired across owner-scoped PRs.

## Cross-Mgr coordination (informational; NOT a same-slice blocker)

The umbrella `bridge_source_span_file_participation_retired` ratchet (Verification's `bridge_retirement_ledger_zero` audit per `r3-v-bridge-retirement-ledger-zero-audit.md`) cannot flip green on this PR alone — production sites in `lens_apply.rs` / `lower.rs` / `emit.rs` (per `r3-structure.md:115`) remain post-this-PR. Verification's ledger-zero audit advances when ALL audit-packet rows retire; this PR contributes rows #2 + #6 progress only. No same-slice ratchet-test gate applies to this PR; ping Verification Mgr (#2075 / lane #1940) on merge so they can update the ledger-zero audit progress field, but their action is **post-merge tracking**, not a pre-merge blocker.

## Scope (Substrate-owned, narrow)

Per **r3-program-plan.md §5 Y4 scope-clarification**:

> Substrate-owned: `SourceSpan.file` participation checks (**hand-Rust audit sites only** —
> `bootstrap.rs:519` doc-comment + `:137` / `:287` / `:309` hardcoded path strings;
> codegen-emitted `SourceSpan::new(...)` offsets in `bootstrap_generated.rs` are a
> separate generated-file bridge shape, **NOT counted as Substrate-owned manual hand-Rust**).

This brief targets **bootstrap.rs hand-Rust sites only**. The 19-row audit packet's broader rows (`lens_apply.rs`, `test_runner.rs`, `dag.rs::declaration_by_name` rank tables, etc.) are owned by Verification / PB per the audit-packet leaf-first schedule — **not in scope here**.

## Concrete anchor sites in `bootstrap.rs` at HEAD

| Anchor | Site | Bridge shape |
|---|---|---|
| 1 | `:125-129` Bool resolver | `d.span.file == BOOL_TYPES_FILE` — identity-by-path-string (audit row #2) |
| 2 | `:138` | `SourceSpan::new(BOOL_TYPES_FILE, 0, 0)` — diagnostic span manufactured from path constant (audit row #2) |
| 3 | `:285` | `BootstrapAuthorityKey::new(PIPELINE_AUTHORITY_FILE)` — string-keyed authority (audit row #6) |
| 4 | `:288` | `SourceSpan::new(PIPELINE_AUTHORITY_FILE, 0, 0)` — diagnostic span (audit row #6) |
| 5 | `:309` | `SourceSpan::new("<test:realization>", 0, 0)` — **test fixture, NOT a bridge** (synthetic span; OK to keep) |
| 6 | `:519` doc-comment | references `SourceSpan.file` for prose only (no runtime check; doc-update on retirement) |

Authority constants: `BOOL_TYPES_FILE` (`dsl/std/types.dag`), `PIPELINE_AUTHORITY_FILE` (`src/v3/compiler/src/pipeline.dag` per `pipeline_authority.rs`).

## Retirement shape (per audit-packet rows #2 + #6)

**Row #2 (kernel Bool patch)** — replace identity-by-path with structural lookup:
- Use `Dag::declaration_by_name("Bool")` already-canonical lookup; gate by **bootstrap-module witness** (declaration's owning module-id rather than `span.file` string match).
- Diagnostic span: synthesize from `BootstrapAuthorityKey` rather than `SourceSpan::new(BOOL_TYPES_FILE, 0, 0)` — the `BootstrapAuthorityKey::new(...)` wrapper already exists at `:125`, and `DiagnosticAttribution::BootstrapAuthority` (per `:519` doc-comment) is the steady-state attribution surface (PB row 82).
- **Prerequisite**: `dsl/std/types.dag` ↔ `src/v3/std/types.dag` duplicate-module convergence per ROADMAP T-P0 must NOT regress; if both hold a `Bool`, `declaration_by_name` rank still applies (audit row #14 — root blocker, **out-of-scope here**).

**Row #6 (pipeline authority)** — replace `PIPELINE_AUTHORITY_FILE` guards:
- `report_pipeline_authority_error` (`:283-292`) — drop the path-string from both `BootstrapAuthorityKey` and `SourceSpan::new`. Replace with a **`BootstrapAuthority::Pipeline` typed key** (extend the enum if needed); diagnostic span sourced from the offending stage binding's actual `SourceSpan`, not a manufactured `(file, 0, 0)`.
- Coordinate with PB-owned `bridge_include_str_side_channels_retired` (audit row #6 sibling) — `pipeline_authority.rs::ordered_pipeline_stages` already reads `PipelineStageBinding` structurally; the bridge is ONLY in the diagnostic-span manufacturing path.

## Acceptance

1. `bootstrap.rs` no longer references `BOOL_TYPES_FILE` or `PIPELINE_AUTHORITY_FILE` outside doc-comments.
2. The two diagnostic paths (kernel Bool not-found, pipeline-authority error) carry typed `DiagnosticAttribution::BootstrapAuthority` with the appropriate `BootstrapAuthorityKey` variant — verified by `kernel_bool_path_a_diagnostic_carries_bootstrap_authority_attribution` (already exists at `:519+`) extended for the pipeline path if not present.
3. **Ledger discipline (per Ledger-discipline preamble):** `src/v3/std/bridge_ledger.dag` row stays `Open` — do NOT mutate. Audit-packet table at `docs/briefs/bridge-retirement-audit-sourcespan-family.md` updated to mark **rows #2 + #6 only** retired with this PR's # citation (rows #1, #3-5, #7-19 remain under their owners; umbrella row advances to `Retired` only when ALL production sites are retired across owner-scoped PRs).
4. `dag.rs::bridge_source_span_file_participation_retired` ratchet test passes (authored by Verification per the **Cross-Mgr prerequisite** gate above; confirmed-present at HEAD before this PR merges).
5. Bootstrap regen: `cargo test -p v3-compiler bootstrap_regen_fresh -- --ignored` clean (path-string deletion must not perturb regen byte-snapshot).
6. Full suite: `cargo test --workspace --exclude v2-compiler-tests` green; `cargo clippy --all-targets -- -D warnings` clean.

## STOP / PING criteria

- **STOP** if removing `d.span.file == BOOL_TYPES_FILE` (anchor #1) causes `declaration_by_name("Bool")` to resolve to a different declaration (rank-table ambiguity from duplicate `Bool` in `src/v3/std/types.dag` vs `dsl/std/types.dag`). This is audit row #14 root-blocker territory — surface to Mgr; do NOT delete the rank table to "fix it".
- **STOP** if `BootstrapAuthority` enum doesn't have a `Pipeline` variant and adding one cascades into emit/diagnostic surfaces beyond bootstrap.rs. Surface scope-creep to Mgr.
- **PING** Verification Mgr (#1940 / `witty-swift-269` if active) when this lands so they can advance the ledger-zero audit (`docs/briefs/r3-v-bridge-retirement-ledger-zero-audit.md` row 1).

## Cross-Mgr handoff

- **PB Mgr**: audit row #6's sibling (`bridge_include_str_side_channels_retired` for `pipeline_authority.rs` compile-body drift) is PB-owned; this Substrate slice does NOT touch the include_str path, only the diagnostic-span path. No cross-PR coordination needed unless audit row interpretation drifts at execution time.
- **Verification Mgr**: ratchet authoring + ledger-zero audit advancement.

## Worker disposition

Single PR (~150-300 LoC delta + test). Use standing-authority merge per Director directive 2026-05-07 once CLEAN + green CI + reviewer comments without BLOCKING + sitting >30min.

— Authored by warm-wolf-698 (Substrate Mgr) 2026-05-07 per Director endorsement at gunbc#828 #issuecomment-4394293399.
