# v0.1.0 — Maintainer-Facing Release State

Maintainer-facing state snapshot for the v0.1.0 review (target: June 1). The
pre-release punch list remains [`RELEASE_TODO.md`](../RELEASE_TODO.md); this
doc summarizes accumulated decisions and gates so a reviewer can decide
whether the tag is ready without re-deriving status from git log.

This doc is **private** (stripped from the public snapshot — see
[Item G](#item-g--this-doc-is-private)). The user-facing GitHub Release body
is authored separately at `docs/release/v0.1.0-release-notes.md` (in flight as
`adhoc-a9231edc-66e`) and is what gets pasted into the GitHub Release form at
tag time.

Snapshot date: 2026-05-30.

## Scope revision (2026-05-30, reviewer-driven)

The external reviewer's posture is now the working frame for v0.1.0:

> "Do not release everything we have. Release only a small, verified,
> fail-closed product surface. Everything else stays private, stripped, or
> explicitly unsupported."

The rest of this doc adopts that posture. Anything outside the explicitly
verified surface either stays private or is documented as unsupported with a
fail-closed runtime behavior. The previously broad "ship daglang + gunbc"
scope from earlier drafts is narrowed by the D-REL decisions below.

## Goals / Non-goals (under the revised scope)

**Goals.**

1. First public tag of **daglang** on `gunb-ai/daglang`, scoped to the
   verified language subset and example surface (see D-REL-3).
2. **gunbc** v2 self-hosted compiler binary distributed via GitHub Releases
   on targets that pass a verified release dry-run (see D-REL-2).
3. User-facing public docs only — `README`, `LICENSE`, `CHANGELOG`,
   `GETTING_STARTED`, `LANGUAGE`/`SYNTAX`, `CLI`, `EXAMPLES`, `SUPPORTED`,
   and (optionally) `CONTRIBUTING` — per D-REL-4.
4. Public/private split landed: `gunb-ai/daglang` public, `gunb-ai/gunbc`
   private scratchpad; one-shot seed via `scripts/publish-snapshot.sh`,
   sync direction inverts after v0.1.0.
5. Public website at <https://gunb.ai> from GitHub Pages on
   `gunb-ai/daglang` (PR #1 on `daglang`, session `fierce-dove-549`,
   maintainer flips on launch day).

**Non-goals.**

1. v4 substrate in the public v0.1.0 snapshot (D-REL-1; reviewer
   recommendation = strip).
2. Comprehensive `.dag` comment-stripping pass — load-bearing markers
   (`🟡 dissolve-on-arrival`, `🟢`/`🔴` coproduct tags, `// Anchor:`,
   dissolve-target session-slug attribution, `adhoc-<UUID>` work-item refs)
   are NOT cleanup targets.
3. Inverted-sync tooling (private → public PR flow); v0.2.0+ concern.
4. External community surface (Issues/Discussions wiring on `daglang`).
5. Removal of `src/v3/` from the internal workspace (stripped from snapshot
   only).
6. Frontend ([`gunb-ai/frontend`](https://github.com/gunb-ai/frontend)) —
   separate repo, own release cadence, does not gate v0.1.0.
7. Any binary target that does not pass the release dry-run end-to-end
   (D-REL-2 — drop it from the matrix, do not ship as "supported").

## D-REL decisions

DECIDED items are the working default. PENDING items carry the reviewer
recommendation as the default until the project maintainer rules otherwise.

| ID | Topic | Reviewer recommendation | Status |
|----|-------|-------------------------|--------|
| D-REL-1 | v4 in public v0.1.0 | **Strip `src/v4` from public snapshot.** | PENDING maintainer confirmation. Working default: strip. |
| D-REL-2 | Binary distribution scope | **Ship only verified targets; drop any that fail dry-run.** | PENDING maintainer confirmation. Prior call was "all 6 are blockers" — reconciliation needed. |
| D-REL-3 | Day-one supported language subset | **Small example-backed subset, anchored to `weather.dag` + `interp_test.dag` and the `dsl/std` vocabulary they use; unsupported features fail-closed.** | PENDING maintainer confirmation. `docs/SUPPORTED.md` authoring is downstream. |
| D-REL-4 | Public docs list | **Ship only user docs: `README`, `LICENSE`, `CHANGELOG`, `docs/GETTING_STARTED.md`, `docs/LANGUAGE.md` (or `SYNTAX.md`), `docs/CLI.md`, `docs/EXAMPLES.md`, `docs/SUPPORTED.md`, `docs/CONTRIBUTING.md` (only if public PRs are wanted); strip all other docs.** | PENDING maintainer confirmation. |
| D-REL-5 | Release before v4 confidence | **YES, conditional on D-REL-1 = strip-v4.** | PENDING maintainer confirmation. |

## Already-decided rulings (apply throughout the doc)

- **GitHub plan migration:** DONE 2026-05-30, Enterprise → Teams (org).
  Post-migration CI smoke still to confirm.
- **Distribution channels in scope:** Homebrew tap, `.deb`, and APT are
  IN scope for v0.1.0 (the project maintainer reversed the earlier defer on
  2026-05-30; `src/v4/install/install.dag` carries 🟡 markers for these as
  active emission targets).
- **Public website:** GitHub Pages from `gunb-ai/daglang`, served at
  <https://gunb.ai>. The `daglang` PR #1 (session `fierce-dove-549`) is
  ready; the visibility/Pages flip is a launch-day maintainer action.
- **Private ↔ public sync model:** public `gunb-ai/daglang` is the source
  of truth post-launch; private `gunb-ai/gunbc` is a scratchpad whose sole
  purpose is to keep internal session traffic off the public repo. The
  sync direction inverts at the v0.1.0 tag: one-shot force-push seed +
  visibility flip at tag time, then v0.2.0+ flows reverse (public PRs
  primary, private pulls from public).
- **Dissolution comments stay.** `🟡 dissolve-on-arrival` markers,
  dissolve-target session-slug attribution, and `adhoc-<UUID>` work-item
  refs are all load-bearing model marks and are NOT cleanup targets. They
  ship in the public snapshot as-is.
- **No PM jargon in published artifacts.** Phrasings like
  "operator-ratified", "operator directive", "operator decided", "per the
  operator", and any `T-##` / session-ID / dashboard / audit / scratchpad
  machinery are out of the public snapshot. This doc uses neutral phrasing
  ("the project maintainer", "you", or passive voice) throughout, and the
  remaining root docs (`README`, `THESIS`, `INVARIANTS`, etc.) have
  already been audited per the earlier §3 cleanup.

## Item D — `SUPPORTED.md` (the heart of v0.1.0)

`docs/SUPPORTED.md` is a **separate-file deliverable**, authored downstream
of D-REL-3. When written, it will enumerate the verified product surface as
the single normative answer to "what does v0.1.0 support":

- **Supported language subset** — the exact set of `.dag` constructs that
  v0.1.0 compiles and runs end-to-end. Anchored to the examples that ship
  (`weather.dag`, `interp_test.dag`) and the `dsl/std` vocabulary those
  examples exercise. Anything not on this list is unsupported.
- **Verified install targets** — every OS/arch combination that passed the
  release dry-run (per D-REL-2). Targets that did not pass are absent;
  they are not listed as "experimental".
- **CLI commands** — the documented `gunbc` subcommands and flags that are
  on the support contract.
- **OS support matrix** — concrete OS+arch+libc combinations, not "Linux".
- **Fail-closed guarantee** — the runtime/compiler explicitly refuses
  features outside the supported subset rather than partially executing or
  silently no-op'ing. This is the central reviewer ask: unsupported ≠
  undefined behavior.

## Item E — Acceptance gates (replaces prior acceptance criteria)

The acceptance criteria from earlier revisions of this doc are superseded
by the reviewer's Gates A–E. The tag does not happen until all five are
green.

**Gate A — Product confidence.**

- Fresh-checkout build succeeds.
- `gunbc --help` and each subcommand's `--help` render correctly.
- Every example documented in the public docs runs successfully end-to-end.
- Every unsupported feature exercised in test fails closed (no partial
  output, no silent no-op).
- No public command is documented as supported without an end-to-end test
  backing it.

**Gate B — Scope hygiene.**

- Public `SUPPORTED.md` exists and is authoritative.
- Public `README` states the v0.1.0 scope on the first screen.
- `src/v4` is stripped from the public snapshot, or — if it ships — it is
  conspicuously absent from public support claims.
- No public doc references `T-##` / operator / dashboard / session /
  audit / scratchpad machinery.

**Gate C — Install.**

- Build/install instructions are verified on each advertised target.
- Unverified targets are removed from `release.yml`'s matrix, not shipped
  as "best effort".
- Package-manager installs (Homebrew, `.deb`, APT) are either present and
  verified, or explicitly labeled post-v0.1.0 in the public docs.

**Gate D — Export sanitation.**

- `scripts/publish-snapshot.sh` dry-run is inspected by hand.
- A `public-export-manifest` is generated (full file list of the exported
  tree).
- Every path on the strip list is absent from the export.
- `grep` for private/internal terms (`operator-ratified`, `T-##`, session
  slugs, `adhoc-`, dashboard URLs) is clean.
- `RELEASE_v0.1.0.md` and `RELEASE_TODO.md` are NOT in the public export
  (see [Item G](#item-g--this-doc-is-private)).

**Gate E — Release mechanics.**

- Release-artifact dry-run succeeds for every advertised target.
- Checksums are generated and published alongside the artifacts.
- A fresh clone of the public repo after the seed push matches the export
  commit byte-for-byte.
- The release notes (in `docs/release/v0.1.0-release-notes.md`) match the
  support matrix in `SUPPORTED.md` — no claim ships that's not in
  `SUPPORTED.md`.

## Item F — Rollback plan

If the publish leaks private content (any file from the strip list, any
PM/session jargon, any unintended path):

1. **Immediately flip `gunb-ai/daglang` back to PRIVATE.**
2. Delete the release and the tag if either has been published.
3. Rotate any credentials that were exposed (tokens, keys, webhook
   secrets) — even if the leak window was short.
4. Fix the strip list (`scripts/publish-snapshot.sh` `STRIP_PATHS`) or the
   root-doc source so the leaked content cannot leak again.
5. Re-run `scripts/publish-snapshot.sh` dry-run and the public-clone smoke
   test. Only re-attempt publish once **Gate D** is green again.

## Item G — This doc is private

`docs/RELEASE_v0.1.0.md` is added to `scripts/publish-snapshot.sh`
`STRIP_PATHS` in this PR, alongside `RELEASE_TODO.md` and `WISHLIST.md`
(maintainer-facing planning docs that must not ship).

> **Collision note.** `adhoc-12a071f5-04a` is a separate in-flight cleanup
> PR adding `RELEASE_TODO.md` and `WISHLIST.md` to `STRIP_PATHS`. Whichever
> PR lands first wins; the other rebases on top. This PR adds all three to
> be safe — if `adhoc-12a071f5-04a` lands first, the conflict is a trivial
> merge.

## Item H — User-facing release notes (separate artifact)

The text the project maintainer pastes into the GitHub Release form at tag
time lives at `docs/release/v0.1.0-release-notes.md` (in flight as
`adhoc-a9231edc-66e`). That file is user-facing and is shaped by
`SUPPORTED.md`. This doc (`RELEASE_v0.1.0.md`) is maintainer-facing and is
out of the public snapshot.

## Section-by-section status (against `RELEASE_TODO.md`)

### §0 — Merge gate

- [x] PR #3826 (`v2-compiler` → `gunbc` rename) — merged (`ddfc4fbf7`).
- Clean-checkout build of `target/release/gunbc` — folded into **Gate A**.

### §1 — GitHub plan migration

DONE 2026-05-30 (Enterprise → org/Teams). Pre-flight audit receipt:
[`docs/admin/github-enterprise-to-teams-audit-2026-05-29.md`](admin/github-enterprise-to-teams-audit-2026-05-29.md).
Post-migration CI smoke remains as a one-shot maintainer check.

### §2 — Public/private repo model

See the decided sync model in **Already-decided rulings** above. Landed
mechanics: `public` remote → `git@github.com:gunb-ai/daglang.git`;
`scripts/publish-snapshot.sh` implements strip-list + dry-run default +
`PUBLISH_CONFIRM=yes` guard for the destructive force-push; `_internal/`
carries `INVARIANTS_OPS.md`, `ROADMAP_OPS.md`,
`DOWNSTREAM_REQUIREMENTS.md`; the publish script still strips by explicit
path list rather than relying on `_internal/` alone.

Open implementation questions for v0.2.0+ (do not block v0.1.0): tooling
for the inverted flow; trigger policy for private→public promotion;
external community surface (Issues + Discussions wiring).

### §3 — Root doc cleanup

`README.md`, `THESIS.md`, `INVARIANTS.md`, `ROADMAP.md` rewritten with the
v4 framing block and external-reader language; internal session IDs and
provenance jargon no longer appear in their public bodies. Long-form
rationale moved to `docs/invariants/` and `docs/thesis/`. Operational
ROADMAP/INVARIANTS material lives in `_internal/ROADMAP_OPS.md` and
`_internal/INVARIANTS_OPS.md`.

Note: under D-REL-4 the public docs list narrows further; root docs
beyond the user-facing set (`THESIS`, `INVARIANTS`, `MODELING`, `CODING`,
`TESTING`) are stripped from the public export.

### §4 — Comment stripping from code files

**Not a v0.1.0 blocker.** The load-bearing markers (`🟡 dissolve-on-arrival`,
`🟢`/`🔴` coproduct tags, `// Anchor:`, dissolve-target session-slug
attribution, `adhoc-<UUID>` work-item refs) are not cleanup targets per
the decided rulings above. Verbosity in `src/v2/05_emit_rust.dag` and v2
core files is cosmetic and does not block the tag.

### §5 — Binary distribution

- `src/v4/workflow/release.dag` — present (semantic authority for the
  target matrix lives here).
- `src/v4/install/install.dag` — present, carries 🟡 markers for Homebrew,
  `.deb`, and APT as active emission targets.
- `.github/workflows/release.yml` — present.
- **Remaining for tag:** end-to-end dry-run of `release.yml` against a
  throwaway pre-tag. Per D-REL-2, any target that fails the dry-run is
  **dropped from the matrix**, not shipped as "best effort". Homebrew,
  `.deb`, and APT are IN scope for v0.1.0 (per the decided ruling above).

### §6 — Housecleaning

- `src/v1/` — deleted.
- `src/v3/` — still in tree, stripped from the public snapshot.
- `wip/chatgpt_reviewer.dag` — stripped from snapshot.
- `.cursor/` and `_internal/` — stripped from snapshot.
- `Cargo.toml` workspace metadata for crates.io readiness — outstanding;
  only required for Phase 4 (`cargo install`), not the tag.
- Public `PULL_REQUEST_TEMPLATE.md` and public `.github/` workflow trim
  (`ci-spot-rerun.yml`, `tier3-baseline-capture.yml`) — outstanding;
  reviewer call whether they block the tag or follow in v0.1.1.

## Tagging procedure

When **Gates A–E** are all green:

1. Tag `v0.1.0` on internal `main`.
2. Run `PUBLISH_CONFIRM=yes scripts/publish-snapshot.sh --publish` as the
   one-shot launch seed.
3. Flip `gunb-ai/daglang` PRIVATE → PUBLIC.
4. Enable GitHub Pages on `gunb-ai/daglang` (per the daglang PR #1
   maintainer action).
5. Verify the fresh public clone matches the export commit byte-for-byte.
6. Sync direction inverts from this point onward (see the decided
   ruling on the sync model above).

## Cross-refs

- Pre-release punch list: [`RELEASE_TODO.md`](../RELEASE_TODO.md) (private)
- Wishlist / deferred ideas: [`WISHLIST.md`](../WISHLIST.md) (private)
- User-facing release notes: `docs/release/v0.1.0-release-notes.md`
  (in flight, separate PR)
- Publish mechanism: [`scripts/publish-snapshot.sh`](../scripts/publish-snapshot.sh)
- GH plan audit receipt: [`docs/admin/github-enterprise-to-teams-audit-2026-05-29.md`](admin/github-enterprise-to-teams-audit-2026-05-29.md)
- Release workflow: [`.github/workflows/release.yml`](../.github/workflows/release.yml)
- Release model authority: [`src/v4/workflow/release.dag`](../src/v4/workflow/release.dag)
- Install model authority: [`src/v4/install/install.dag`](../src/v4/install/install.dag)
- Frontend repo (separate, not part of this tag): [`gunb-ai/frontend`](https://github.com/gunb-ai/frontend)
- Public website source: `gunb-ai/daglang` PR #1 (session `fierce-dove-549`)
