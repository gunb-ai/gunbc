# v0.1.0 — Consolidated Release State

State snapshot for v0.1.0 review (target: June 1). Source of truth for the
pre-release punch list is [`RELEASE_TODO.md`](../RELEASE_TODO.md); this doc
summarizes where each section stands so a reviewer can decide whether the tag
is ready without re-deriving status from git log.

Snapshot date: 2026-05-30.

## Goals / Non-goals / Acceptance criteria

**Goals (what v0.1.0 ships).**

1. First public tag of **daglang** — the language surface plus `dsl/std`
   and `extdeps` vocabulary — on `gunb-ai/daglang`.
2. **gunbc** v2 self-hosted compiler binary distributed via GitHub Releases
   for the six-target matrix in [`src/v4/workflow/release.dag`](../src/v4/workflow/release.dag).
3. External-reader root docs (`README`, `THESIS`, `INVARIANTS`, `MODELING`,
   `CODING`, `TESTING`) published as the public-facing surface.
4. Public/private split landed: `gunb-ai/daglang` public, `gunb-ai/gunbc`
   private scratchpad; one-shot seed via `scripts/publish-snapshot.sh`.

**Non-goals (explicitly NOT in v0.1.0).**

1. v4 substrate as the production pipeline — v4 ships as published source
   for transparency only; v2 remains the active compiler.
2. Homebrew tap, `.deb`/APT, `cargo install` distribution paths (Phases 2–4,
   post-tag).
3. Comprehensive `.dag` comment-stripping pass (§4 deferred; load-bearing
   markers make a blind pass unsafe).
4. Inverted-sync tooling (private → public PR flow); v0.2.0+ concern.
5. External community surface (Issues/Discussions wiring on `daglang`).
6. Removal of `src/v3/` from the internal workspace (stripped from snapshot
   only).

**Acceptance criteria (testable conditions for `git tag v0.1.0`).**

- [x] PR #3826 (`v2-compiler` → `gunbc` rename) merged.
- [x] GitHub plan audit complete — no Enterprise-only feature in use.
- [x] GitHub plan downgraded Enterprise → org (Teams) by operator.
- [x] `public` git remote points at `git@github.com:gunb-ai/daglang.git`.
- [x] `scripts/publish-snapshot.sh` implements strip-list + dry-run default
      + `PUBLISH_CONFIRM=yes` guard.
- [x] Public/private sync model decided and documented (§2).
- [x] Root docs rewritten with external-reader framing; internal session
      IDs and operator-ratified provenance removed from public bodies.
- [x] `src/v4/workflow/release.dag`, `src/v4/install/install.dag`,
      `.github/workflows/release.yml` all present.
- [ ] Clean-checkout build of `target/release/gunbc` succeeds.
- [ ] `release.yml` dry-run on a throwaway pre-tag produces all six target
      artifacts.
- [ ] Final `scripts/publish-snapshot.sh` dry-run inspected: no stripped
      paths, internal session IDs, or operator-ratified provenance leak
      into the export commit.
- [ ] Reviewer sign-off that §4 comment-stripping and §6 PR-template /
      workflow-trim items are acceptable as v0.1.1 follow-ups (or a focused
      PR addresses them first).
- [ ] Operator ready to flip `gunb-ai/daglang` PRIVATE → PUBLIC immediately
      after the seed push.
- [ ] `v0.1.0` tagged on internal `main`, seed `--publish` run, visibility
      flipped, public HEAD verified to match export.

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
Enterprise runner groups, `enterprise:` workflow keys all clear). Plan
downgrade Enterprise → org (Teams) executed by operator 2026-05-30.
Post-migration CI smoke remains outstanding (operator-driven, not a code
change).

### §2 — Public/private repo model

**Decided model (post-launch steady state).** `gunb-ai/daglang` is the
source of truth. `gunb-ai/gunbc` remains private and serves as a development
scratchpad whose sole purpose is to keep internal session traffic (agent
briefs, debt ledgers, postmortems, dashboard ops) off the public repo.

**Sync direction inverts at the v0.1.0 tag.**

- *v0.1.0 path (now → tag):* substantive work continues to merge into the
  private `main` (`gunb-ai/gunbc`). One more dry-run of
  [`scripts/publish-snapshot.sh`](../scripts/publish-snapshot.sh) seeds the
  launch snapshot. At tag time, `gunb-ai/daglang` flips PRIVATE → PUBLIC.
  The force-push internal → public is a one-shot launch seed; external
  contributors cannot meaningfully open PRs against `daglang` before
  v0.1.0 lands because that force-push wipes any history they would have
  branched from.
- *v0.2.0+ path:* substantive code PRs target `gunb-ai/daglang` directly.
  Private `gunb-ai/gunbc` pulls from public `main` to stay in sync. The
  §2 strip-list paths (`docs/briefs`, `docs/history`, `docs/debt`,
  `docs/review-findings`, `docs/admin`, `docs/db-history`, `docs/postmortems`,
  `docs/audit`, `docs/r3`, `docs/proposals`, `docs/perf`, `docs/decisions`,
  `src/v3`, `src/v4/{TASKS,BRIEF_TEMPLATE,CULTURE}.md`, `wip`,
  `scripts/session-dashboard`, `tools/gen_gunbc_ci_workflow_dag`, `.cursor`)
  stay private only.

**Landed mechanics.** `public` git remote configured →
`git@github.com:gunb-ai/daglang.git`. Publish script implemented with
strip-list, dry-run default, and `PUBLISH_CONFIRM=yes` guard for the
destructive force-push. `_internal/` exists and carries
`INVARIANTS_OPS.md`, `ROADMAP_OPS.md`, `DOWNSTREAM_REQUIREMENTS.md`; the
publish script still strips by explicit path list rather than relying on
`_internal/` alone.

**Open implementation questions for v0.2.0+ (do not block v0.1.0):**

1. Tooling for the inverted flow — how private `gunb-ai/gunbc` opens PRs
   *against* public `gunb-ai/daglang` and merges results back into the
   private scratchpad without losing the private-only directories.
2. Trigger policy — what causes a private-side change to be promoted to a
   public PR (per-commit, batched, manual operator gesture).
3. External community surface — where bugs and discussions are filed
   (likely Issues + Discussions on `gunb-ai/daglang`, but not yet wired).

**Remaining for tag:** `v0.1.0` tag itself; the seed `--publish` run; the
private→public visibility flip on `gunb-ai/daglang`.

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

1. ~~GitHub plan downgrade executed.~~ Done 2026-05-30 (Enterprise → org/Teams);
   post-migration CI smoke still to confirm.
2. `release.yml` dry-run produces all six target artifacts on a throwaway tag.
3. Reviewer sign-off that §4 (comment stripping) and the §6 PR-template /
   workflow-trim items are acceptable as v0.1.1 follow-ups, OR a focused PR
   addresses them first.
4. Final `scripts/publish-snapshot.sh` dry-run inspected by reviewer; no
   leakage of stripped paths, internal session IDs, or operator-ratified
   provenance into the export commit.
5. Operator ready to flip `gunb-ai/daglang` PRIVATE → PUBLIC immediately
   after the seed push (per §2 decided model).

When 1–5 are green, tag `v0.1.0` on internal `main`, run
`PUBLISH_CONFIRM=yes scripts/publish-snapshot.sh --publish` as the one-shot
launch seed, flip `gunb-ai/daglang` to public, and verify the public repo
HEAD matches the export. Sync direction inverts from that point onward
(see §2).

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
