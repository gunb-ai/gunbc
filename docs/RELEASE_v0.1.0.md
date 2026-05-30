# v0.1.0 — Consolidated Release State

State snapshot for v0.1.0 review (target: June 1). Source of truth for the
pre-release punch list is [`RELEASE_TODO.md`](../RELEASE_TODO.md); this doc
summarizes where each section stands so a reviewer can decide whether the tag
is ready without re-deriving status from git log.

Snapshot date: 2026-05-30.

## What v0.1.0 is

The first public tag of **daglang** (language + `dsl/std`/`extdeps` vocabulary)
together with **gunbc** (the self-hosted v2 compiler that validates `.dag` and
emits Rust/Python/Go). The v4 substrate rewrite is in-tree but not the v0.1.0
deliverable — it ships as published source for transparency, not as the
production pipeline. The README, THESIS, INVARIANTS, MODELING, CODING, and
TESTING root docs define the public-facing surface; the published snapshot is
produced by [`scripts/publish-snapshot.sh`](../scripts/publish-snapshot.sh)
against the `public` remote (`gunb-ai/daglang`).

## Section-by-section status (against `RELEASE_TODO.md`)

### §0 — Merge gate

- [x] PR #3826 (`v2-compiler` → `gunbc` rename) — merged (`ddfc4fbf7`). All
      downstream renames (`gunbc-dag` → `gunbc-app`, lens registry smoke,
      ci-workflow.dag) followed.
- Clean-checkout build of `target/release/gunbc` — to be confirmed as part of
  the §5 release.yml dry-run before tagging.

### §1 — GitHub plan migration (Enterprise → Teams)

All pre-flight audits complete 2026-05-29; receipt
[`docs/admin/github-enterprise-to-teams-audit-2026-05-29.md`](admin/github-enterprise-to-teams-audit-2026-05-29.md).
No Enterprise-only feature in use (SAML, audit-log API, IP allowlist, required-2FA,
Enterprise runner groups, `enterprise:` workflow keys all clear). Plan downgrade
itself (billing action) and post-migration CI smoke remain outstanding — these
are operator-driven, not code changes.

### §2 — Public/private repo split

- `public` git remote configured → `git@github.com:gunb-ai/daglang.git`
  (Option A taken: internal slug retained, public uses `daglang`).
- [`scripts/publish-snapshot.sh`](../scripts/publish-snapshot.sh) implemented,
  with strip-list, dry-run default, and `PUBLISH_CONFIRM=yes` guard for the
  destructive force-push.
- `_internal/` exists and carries `INVARIANTS_OPS.md`, `ROADMAP_OPS.md`,
  `DOWNSTREAM_REQUIREMENTS.md` — the "move ops content out of root" item
  is partially done; the publish script still strips by explicit path list
  rather than relying on `_internal/` alone.
- **Remaining:** `v0.1.0` tag (Phase 1a + 1b prerequisites below); first
  real `--publish` run.

### §3 — Root doc cleanup

- `README.md`, `THESIS.md`, `INVARIANTS.md`, `ROADMAP.md` rewritten with
  the v4 framing block and external-reader language. Internal session IDs
  and `T-##`/operator-ratified provenance no longer appear in their public
  bodies; long-form rationale moved to `docs/invariants/` and `docs/thesis/`.
- Operational ROADMAP and INVARIANTS material lives in
  `_internal/ROADMAP_OPS.md` and `_internal/INVARIANTS_OPS.md`.

### §4 — Comment stripping from code files

Largely deferred. `src/v2/05_emit_rust.dag` still carries ~235 standalone
comment lines; v2 `00_core.dag` / `02_parse.dag` / `04_infer.dag` and v4
status markers in `src/v4/std/`, `src/v4/extdeps/`, `src/v4/compiler/`,
`src/v4/lens/` have not had a dedicated stripping pass. **Not a v0.1.0
blocker** unless reviewer rules otherwise: the load-bearing markers
(`🟡 feature:`, `🟢`/`🔴` coproduct tags, `// Anchor:`) make a blind
sed pass unsafe, and the published code remains correct — only verbosity
is at issue.

### §5 — Binary distribution

- `src/v4/workflow/release.dag` — present (semantic authority for the
  six-target matrix lives here).
- `src/v4/install/install.dag` — present.
- `.github/workflows/release.yml` — present (`v*` tag push, musl via `cross`,
  native darwin + windows runners).
- **Remaining for tag:** end-to-end dry-run of `release.yml` against a
  pre-tag (`v0.1.0-rc.0` or equivalent) on a throwaway tag to confirm all
  six artifact uploads succeed; then tag `v0.1.0`.
- Homebrew tap, apt/deb, `cargo install` paths are all scoped post-tag
  per the §7 timeline.

### §6 — Housecleaning

- `src/v1/` — deleted (Cargo.toml had marked it archived).
- `src/v3/` — still in tree, stripped from the public snapshot via §2.
- `wip/chatgpt_reviewer.dag` — stripped from snapshot (publish-snapshot
  removes `wip/`).
- `.cursor/` and `_internal` directories — stripped from snapshot.
- `Cargo.toml` workspace metadata (`description`, `repository`, `homepage`,
  `publish = false` audit across member crates) for crates.io readiness —
  **outstanding**; only required for Phase 4 (`cargo install`), not tag.
- Public `PULL_REQUEST_TEMPLATE.md`, public `.github/` workflow trim
  (`ci-spot-rerun.yml`, `tier3-baseline-capture.yml`) — outstanding;
  reviewer call whether they block the tag or follow in a v0.1.1 polish PR.

## Gating items before `git tag v0.1.0`

1. GitHub plan downgrade executed (or explicit reviewer decision to tag on
   Enterprise and downgrade after).
2. `release.yml` dry-run produces all six target artifacts on a throwaway tag.
3. Reviewer sign-off that §4 (comment stripping) and the §6 PR-template /
   workflow-trim items are acceptable as v0.1.1 follow-ups, OR a focused PR
   addresses them first.
4. First `scripts/publish-snapshot.sh` (dry-run) inspected by reviewer; no
   leakage of stripped paths, internal session IDs, or operator-ratified
   provenance into the export commit.

When 1–4 are green, tag `v0.1.0` on `main`, run `PUBLISH_CONFIRM=yes
scripts/publish-snapshot.sh --publish`, and verify the public repo HEAD
matches the export.

## Deferred to post-v0.1.0

- Phase 2 Homebrew tap (`gunb-ai/homebrew-gunbc` + `Formula/gunbc.rb`).
- Phase 3 `.deb` / APT path.
- Phase 4 crates.io publish + workspace metadata audit.
- §4 comprehensive `.dag` comment-stripping pass.
- `src/v3/` removal from the workspace (currently only stripped from the
  public snapshot, not deleted internally).

## Cross-refs

- Pre-release punch list: [`RELEASE_TODO.md`](../RELEASE_TODO.md)
- Publish mechanism: [`scripts/publish-snapshot.sh`](../scripts/publish-snapshot.sh)
- GH plan audit receipt: [`docs/admin/github-enterprise-to-teams-audit-2026-05-29.md`](admin/github-enterprise-to-teams-audit-2026-05-29.md)
- Release workflow: [`.github/workflows/release.yml`](../.github/workflows/release.yml)
- Release model authority: [`src/v4/workflow/release.dag`](../src/v4/workflow/release.dag)
- Install model authority: [`src/v4/install/install.dag`](../src/v4/install/install.dag)
