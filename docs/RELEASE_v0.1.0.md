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

**Working framing (project maintainer, 2026-05-30, updated post-audit):**

> A small public daglang/gunbc release with a verified subset, verified
> docs, verified install path, and **three advertised target surfaces
> matching the three v2 emit paths: Rust, Go, and Python**. TypeScript
> moves to v4 early-support; it is not v0.1.0. Everything else is
> private, unsupported, or post-v0.1.0.

Rust, Go, and Python are framed as three **verified target/artifact
surfaces** corresponding to `src/v2/05_emit_rust.dag`,
`05_emit_go.dag`, and `05_emit_python.dag`. `SUPPORTED.md` says exactly
what each surface means (the supported `.dag` subset projected through
each emitter); see D-REL-3b below.

The release sentence the tag must make true:

> A fresh public user can install/build `gunbc`, run the documented
> examples, get verified Rust, Go, and Python outputs for the supported
> subset, and every unsupported path either is absent from the docs or
> fails closed.

## Goals / Non-goals (under the revised scope)

**Goals.**

1. First public tag of **daglang** on `gunb-ai/daglang`, scoped to the
   verified `.dag` subset and example surface (see D-REL-3a).
2. **gunbc** v2 self-hosted compiler binary, distributed only on
   per-target-verified build targets (D-REL-2), with **Rust, Go, and
   Python as the three advertised target/artifact surfaces** (D-REL-3b)
   — one per v2 emit lens that already exists.
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

1. v4 substrate in the public v0.1.0 snapshot. **D-REL-1 = CONFIRMED:
   strip `src/v4` from public v0.1.0** (project maintainer 2026-05-30).
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
7. Any binary target (of the six in `release.dag`) that does not pass the
   `release.yml` dry-run end-to-end — D-REL-2 drops it from the matrix
   rather than shipping as "supported".
8. Homebrew, `.deb`, and APT distribution channels — **allowed only if
   the install flow is verified before tag; otherwise omitted from public
   docs and tracked for v0.1.1+**. They are modeled with 🟡 markers in
   `install.dag` as v0.2.0+ emission intent, and the realistic default for
   v0.1.0 is "not shipped" because the Formula / deb-control / apt-repo
   content is not yet emitted. See "Distribution ruling" below.
9. Any target/artifact surface beyond the three v2 emit paths
   (Rust, Go, Python). **TypeScript is explicitly v4 early-support, not
   v0.1.0** (maintainer ruling 2026-05-30, post-audit). C++ / LLVM / etc.
   are not v0.1.0 public support (D-REL-3b).
10. **v4-done predicates** (the six in `src/v4/TASKS.md:805–817`) are
    out of scope for the v0.1.0 tag. The maintainer-facing burn-down is
    tracked privately in
    `docs/planning/v4-done-predicate-burn-down-2026-05-30.md`; do not
    conflate gunbc `main` maturity with the public daglang slice.
    Cross-check (per `nimble-crane-490`): 0/6 predicates PROVEN, 5
    YELLOW, 1 GRAY (P6).

## D-REL decisions

DECIDED items are the working default. PENDING items carry the reviewer
recommendation as the default until the project maintainer rules otherwise.

| ID | Topic | Reviewer recommendation | Status |
|----|-------|-------------------------|--------|
| D-REL-1 | v4 in public v0.1.0 | **Strip `src/v4` from public snapshot.** | **CONFIRMED 2026-05-30.** v4 stays private for v0.1.0; correctness ladder is not at public confidence (diagnosis lane still ~7,951 rustc errors for full-tree v4 Rust emit). **Supersedes** the older `RELEASE_TODO.md` §6 "Keep" list for `src/v4/std`, `compiler/`, etc. — that list pre-dates the scope revision and `RELEASE_TODO.md` is itself stripped from the public export. `scripts/publish-snapshot.sh` `STRIP_PATHS` now strips `src/v4` wholesale. |
| D-REL-2 | Binary distribution scope | **Advertised target = passed dry-run. No dry-run = not advertised. Source build is acceptable if binaries are flaky.** | **CONFIRMED 2026-05-30** (project maintainer). |
| D-REL-3a | Day-one daglang subset (source) | Small example-backed `.dag` subset anchored to `weather.dag` + `interp_test.dag` and the `dsl/std` vocabulary those examples exercise. Anything outside this subset is unsupported and must fail closed. | **CONFIRMED 2026-05-30.** Exact list enumerated in `docs/SUPPORTED.md` (downstream). |
| D-REL-3b | Day-one target/artifact matrix | **Rust, Go, and Python only** (one per v2 emit lens). Rust must pass `rustc`/`cargo check` on shipped examples; Go must pass `go build` / `go vet`; Python must compile (`python -m py_compile`) and the documented example must run. **TypeScript moves to v4 early-support**, not v0.1.0. C++/LLVM/etc. are not public v0.1.0 support. | **CONFIRMED + RECONCILED 2026-05-30** (maintainer post-audit ruling). Reflects the existing v2 substrate (`05_emit_rust.dag`, `05_emit_go.dag`, `05_emit_python.dag`); resolves verification gap V1. Per-surface support level (full compile vs example-run vs artifact-only) declared explicitly in `SUPPORTED.md`. |
| D-REL-4 | Public docs list | **Ship only user docs: `README`, `LICENSE`, `CHANGELOG`, `docs/GETTING_STARTED.md`, `docs/LANGUAGE.md` (or `SYNTAX.md`), `docs/CLI.md`, `docs/EXAMPLES.md`, `docs/SUPPORTED.md`, `docs/CONTRIBUTING.md` (only if public PRs are wanted); strip all other docs.** | **DECIDED 2026-05-30; enforcement PENDING.** The user-facing docs above do not all exist yet (downstream authoring work). `scripts/publish-snapshot.sh` `STRIP_PATHS` currently strips only the agent/process subtrees (`docs/briefs`, `docs/debt`, etc.) and v3/v4 — root `THESIS`/`INVARIANTS`/`MODELING`/`CODING`/`TESTING` and the large `docs/thesis/`, `docs/invariants/`, `docs/planning/`, `docs/design-*` trees are **not yet stripped**. A follow-up pass before tag must (a) land the user docs and (b) extend `STRIP_PATHS` to remove everything outside the D-REL-4 keep list. Gate B and Gate D catch the gap if this slips. |
| D-REL-5 | Release before v4 confidence | **YES**, because D-REL-1 strips v4 and v0.1.0 is scoped to the verified v2 / product slice. | **CONFIRMED 2026-05-30.** |

## Pre-tag verification gaps (open)

A release-readiness audit on 2026-05-30 (`nimble-dove-733`, routed via
PM `still-fox-289`) surfaced five gaps that the gates aspire to close
but that no evidence currently backs. Each is a hard block on tag until
resolved.

| # | Gap | Resolution path | Owner / ETA |
|---|-----|-----------------|-------------|
| V1 | ~~TypeScript surface unsubstantiated.~~ | **RESOLVED 2026-05-30:** maintainer ruled v0.1.0 = Rust + Go + Python (the three existing v2 emit paths); TypeScript moves to v4 early-support. D-REL-3b, Goals, Non-goals, framing, and Gates A/E updated accordingly. | RESOLVED. |
| V2 | **`docs/SUPPORTED.md` does not exist.** Item D names it as the single normative authority; README, website, and release notes all derive from it. No file, no owner, no ETA. | Author `docs/SUPPORTED.md` enumerating the D-REL-3a `.dag` subset, the D-REL-3b verified surfaces with per-surface support level, the verified install/target matrix, CLI commands, OS matrix, and fail-closed guarantee. | **PENDING owner.** Likely lane: `nimble-crane-490` (the v4-done/release-sign-off worker who already cross-checked this doc). |
| V3 | **`install.sh` PR #3992 STALLED.** Open, mergeable=MERGEABLE, 0 reviews, 0 CI checks completed; no shepherd. The doc says `curl install.sh` ships only if #3992 lands and verifies before tag. | Route a shepherd to #3992, OR drop `curl install.sh` from the v0.1.0 install path and rely solely on build-from-source. Decision needed before Gate C can pass. | **PENDING owner.** |
| V4 | **Weather demo end-to-end UNVERIFIED.** Gate A requires every example to run end-to-end; the hero `dsl/examples/weather/` path against `--target rust` has not been exercised against a `target/release/gunbc` built from a clean checkout. The README hero invocation uses `--target dag` rather than the verified emit path. | Build `gunbc` from clean checkout; run the weather example with `--target rust`; verify generated Rust passes `cargo check`; record commit SHA + run timestamp in the Evidence column on Gate A. | **PENDING.** Can be done by any worker with a green build slot. |
| V5 | **No verification log.** Gates A–E are aspirational checklists with no "actual run result + evidence link + verification date" column. A reviewer cannot today distinguish "pre-checked" from "untested". | Add an Evidence column to each gate bullet (✓ verified with commit/run link / ⏳ in flight / ✗ not run). This doc's job is reviewer-readable readiness — Evidence is the only way to fulfil it. | **PENDING.** This doc maintained by current author. |

The five gaps map onto the five gates: V1+V4 → Gate A (product
confidence); V2 → Gate B (scope hygiene, the `SUPPORTED.md` line);
V3 → Gate C (install); V5 → all gates (evidence column on every gate).
Gate D and Gate E are unaffected by this audit.

## Already-decided rulings (apply throughout the doc)

- **GitHub plan migration:** DONE 2026-05-30, Enterprise → Teams (org).
  Post-migration CI smoke still to confirm.
- **Distribution ruling (v0.1.0):**
  - GitHub Release artifacts and source build are the v0.1.0 install path.
  - `curl install.sh` ships only if B1's PR #3992 (`install.sh`
    resurrection) lands and verifies before tag. **Status: STALLED**
    (open, mergeable, 0 reviews, 0 completed CI checks as of 2026-05-30).
    Needs a shepherd or drop from the v0.1.0 install path (see
    verification gap V3 above).
  - Homebrew, `.deb`, and APT may ship **only if** their install flows
    are verified before tag. If any package-manager path is not verified,
    it is **omitted from public docs** and tracked for v0.1.1+.
  - The realistic default is "package managers ship in v0.1.1+" because
    the Formula / deb-control / apt-repo content is not yet emitted.
- **Long-term distribution scope (v0.2.0+):** Homebrew Formula,
  `deb-control`, and APT repo are modeled in `src/v4/install/install.dag`
  with 🟡 markers as active emission intent. They ship as the
  `ShellStatic` / `Formula-Static` / `deb-control` / `apt-repo`
  projections actually land. Modeled ≠ shipped.
- **Public website:** GitHub Pages from `gunb-ai/daglang`, served at
  <https://gunb.ai>. The `daglang` PR #1 (session `fierce-dove-549`) is
  ready; the visibility/Pages flip is a launch-day maintainer action.
  The website **must obey the support matrix in `SUPPORTED.md`**: no
  claim of broad language/compiler support; CTA points to supported
  examples and the verified install path only; the website states that
  v0.1.0's verified target surfaces are **Rust, Go, and Python** (the
  three v2 emit paths) and that TypeScript is **v4 early-support, not
  v0.1.0**.
- **Private ↔ public sync model:** public `gunb-ai/daglang` is the source
  of truth post-launch; private `gunb-ai/gunbc` is a scratchpad whose sole
  purpose is to keep internal session traffic off the public repo. The
  sync direction inverts at the v0.1.0 tag: one-shot force-push seed +
  visibility flip at tag time, then v0.2.0+ flows reverse (public PRs
  primary, private pulls from public).
- **Dissolution comments — split rule.**
  - **In source files (`.dag`, `.rs`, etc.):** `🟡 dissolve-on-arrival`
    markers, dissolve-target session-slug attribution, and
    `adhoc-<UUID>` work-item refs are load-bearing model marks. They
    are NOT cleanup targets and ship in the public snapshot as-is.
  - **In user-facing docs (the D-REL-4 list):** session slugs and
    `adhoc-<UUID>` refs look like internal process residue to a public
    user and must be stripped or neutralized. Gate D's grep enforces
    this scoping.
- **No PM jargon in published artifacts.** Phrasings like
  "operator-ratified", "operator directive", "operator decided", "per the
  operator", and any `T-##` / session-ID / dashboard / audit / scratchpad
  machinery are out of the public snapshot. This doc uses neutral phrasing
  ("the project maintainer", "you", or passive voice) throughout, and the
  remaining root docs (`README`, `THESIS`, `INVARIANTS`, etc.) have
  already been audited per the earlier §3 cleanup.

## Item D — `SUPPORTED.md` (the heart of v0.1.0)

`docs/SUPPORTED.md` is a **separate-file deliverable**, authored downstream
of D-REL-3a/3b. It is the **single normative answer** to "what does v0.1.0
support" — public `README`, website, and release notes all derive from it.
When written, it will enumerate:

- **Supported source-language subset (D-REL-3a)** — the exact set of
  `.dag` constructs that v0.1.0 compiles and runs end-to-end. Anchored to
  the examples that ship (`weather.dag`, `interp_test.dag`) and the
  `dsl/std` vocabulary those examples exercise. Anything not on this list
  is unsupported.
- **Target/artifact matrix (D-REL-3b)** — **Rust, Go, and Python only**,
  one per existing v2 emit lens. For each surface, `SUPPORTED.md`
  declares the support level explicitly:
  - *Full compile target* — `.dag` → emitted source → external toolchain
    check passes for the documented examples (`rustc`/`cargo check`,
    `go build`/`go vet`, `python -m py_compile`).
  - *Runnable example target* — at least one example runs end-to-end
    under the emitted target (default: Rust; Go and Python where
    declared in `SUPPORTED.md`).
  - "v0.1.0 supports X" is never used without saying what "supports"
    means.
  - **Out of scope (call out explicitly):** TypeScript (v4
    early-support, not v0.1.0), C++, LLVM, arbitrary corpus emit, v4
    full-tree Rust emit, React app generation, self-host fixed point.
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

**Evidence convention.** Each bullet below should be annotated as it is
verified: `✓ <commit-sha> <YYYY-MM-DD>` when checked end-to-end,
`⏳ <owner>` when in flight, `✗` when not yet attempted. As of
2026-05-30 the gates are aspirational — no bullet has an evidence tag
yet. See "Pre-tag verification gaps" above (V5) for the open work.

**Gate A — Product confidence.**

- Fresh-checkout build succeeds.
- `gunbc --help` and each subcommand's `--help` render correctly.
- Every example documented in the public docs runs successfully end-to-end.
- Every unsupported feature exercised in test fails closed (no partial
  output, no silent no-op).
- No public command is documented as supported without an end-to-end test
  backing it.
- **Rust surface (D-REL-3b):** every documented example emits Rust and
  the generated Rust passes `rustc` / `cargo check` (or `cargo run`
  where the example is runnable).
- **Go surface (D-REL-3b):** every documented example emits Go and the
  generated Go passes `go build` / `go vet`. If an example is not
  supposed to support Go, that absence is listed in `SUPPORTED.md`.
- **Python surface (D-REL-3b):** every documented example emits Python,
  passes `python -m py_compile`, and the documented runnable example
  executes successfully. If an example is not supposed to support
  Python, that absence is listed in `SUPPORTED.md`.
- **Negative tests:** unsupported-feature examples fail closed with
  named diagnostics.

**Gate B — Scope hygiene.**

- Public `SUPPORTED.md` exists and is authoritative.
- Public `README` states the v0.1.0 scope on the first screen.
- `src/v4` is stripped from the public snapshot (D-REL-1 = CONFIRMED;
  no "if it ships" escape).
- No public doc references `T-##` / operator / dashboard / session /
  audit / scratchpad machinery.

**Gate C — Install.**

- Build/install instructions are verified on each advertised target.
- Unverified targets are removed from `release.yml`'s matrix, not shipped
  as "best effort".
- Package-manager installs (Homebrew, `.deb`, APT) are either present
  and verified before tag, or **omitted from public docs and tracked for
  v0.1.1+**. They are never shipped "best-effort".

**Gate D — Export sanitation.**

- `scripts/publish-snapshot.sh` dry-run is inspected by hand.
- A `public-export-manifest` is generated (full file list of the exported
  tree).
- Every path on the strip list is absent from the export.
- `grep` for private/internal terms (`operator-ratified`, `T-##`, session
  slugs, `adhoc-<UUID>` refs, dashboard URLs) is clean **in user-facing
  docs only** — `README`, `LICENSE`, `CHANGELOG`, `docs/GETTING_STARTED.md`,
  `docs/LANGUAGE.md` / `SYNTAX.md`, `docs/CLI.md`, `docs/EXAMPLES.md`,
  `docs/SUPPORTED.md`, `docs/CONTRIBUTING.md`. **Excluded from this grep:**
  all `.dag` source files and any path documented as carrying load-bearing
  model marks (the `🟡 dissolve-on-arrival` markers, dissolve-target
  session-slug attribution, and `adhoc-<UUID>` work-item refs ship in the
  public snapshot as-is per the "dissolution comments stay" ruling above).
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
- Release notes claim **only Rust + Go + Python** support (the three
  v2 emit paths). Any mention of TypeScript is labeled "v4
  early-support, not v0.1.0"; C++ / LLVM / etc. are labeled "not
  supported in v0.1.0" or omitted entirely.

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
`adhoc-a9231edc-66e`). That file is user-facing and **derives directly
from `SUPPORTED.md`** — not from internal release goals, not from
maintainer planning state. If a claim is not in `SUPPORTED.md`, it does
not belong in the release notes. This doc (`RELEASE_v0.1.0.md`) is
maintainer-facing and is out of the public snapshot.

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
`TESTING`) **are intended to be stripped** from the public export, but
`scripts/publish-snapshot.sh` does not yet enforce that — the strip-list
extension is queued as a pre-tag follow-up alongside the user-doc
authoring work (see D-REL-4 status above).

### §4 — Comment stripping from code files

**Not a v0.1.0 blocker.** The load-bearing markers (`🟡 dissolve-on-arrival`,
`🟢`/`🔴` coproduct tags, `// Anchor:`, dissolve-target session-slug
attribution, `adhoc-<UUID>` work-item refs) are not cleanup targets per
the decided rulings above. Verbosity in `src/v2/05_emit_rust.dag` and v2
core files is cosmetic and does not block the tag.

### §5 — Binary distribution

Two separate concerns — keep them decoupled:

**Binary target matrix (v0.1.0).** The six-target matrix in
`src/v4/workflow/release.dag` (musl-linux ×2, darwin ×2, windows-msvc ×2)
is gated per-target by `release.yml` dry-run under D-REL-2: each target
that produces a working artifact ships; each target that fails the dry-run
is **dropped from the matrix** for v0.1.0 (no "best effort" shipping).
`SUPPORTED.md` lists which targets actually shipped.

- `src/v4/workflow/release.dag` — present (semantic authority for the
  six-target matrix lives here).
- `.github/workflows/release.yml` — present.
- **Remaining for tag:** end-to-end dry-run of `release.yml` against a
  throwaway pre-tag; per-target drop decisions made from the dry-run
  results.

**Install/distribution channels (v0.1.0 vs v0.2.0+).** Package-manager
channels are a separate axis from the binary target matrix:

- *v0.1.0 install paths:* `curl install.sh` (pending B1's PR #3992
  `install.sh` resurrection) and/or build-from-source (always works).
  Homebrew, `.deb`, and APT ship **only if** their install flows verify
  before tag (see "Distribution ruling" above). Realistic default: they
  do not ship at v0.1.0 because the Formula / deb-control / apt-repo
  content is not yet emitted.
- *v0.2.0+ scope:* Homebrew Formula, deb-control, and APT repo ship as
  the corresponding `ShellStatic` / `Formula-Static` / `deb-control` /
  `apt-repo` projections actually land. Independent of the per-target
  binary matrix outcome.

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
