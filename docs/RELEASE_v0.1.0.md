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

**ZERO OVERCLAIM (operator 2026-05-30, hard bar).** The reviewer's
"silently emits plausible-looking output" anti-pattern is the
release-blocking failure mode. **No claim ships without evidence.**
This applies across `SUPPORTED.md`, the homepage, the release notes,
and any per-surface support level: if a target is not verified at the
claimed scope (example, fixture, smoke), it does not appear in the
support contract at that scope. Parallel disclaimers landing via
`silent-bee-431` and `sharp-otter-407` carry the same posture —
cross-reference when authoring `SUPPORTED.md` to avoid drift.

**Reconciliation with D-REL-1 (iv) flip (2026-05-30).** "Stripped" is no
longer the default for v3/v4; the operative path is "explicitly
unsupported": v3 and v4 substrate **ships public labeled alpha / WIP**,
sits outside the supported contract (which is Rust + Python + Go only),
and is documented honestly in `SUPPORTED.md` and the
`sharp-otter-407` ship-disposition supplement. The reviewer posture
("only a small, verified, fail-closed product surface" + "everything
else explicitly unsupported") is preserved; the choice between
"stripped" and "explicitly unsupported with alpha label" is now made
case-by-case (process docs / agent traffic = stripped; substrate
in-progress = alpha-labeled).

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

1. ~~v4 substrate in the public v0.1.0 snapshot.~~ **REVISED
   2026-05-30:** v3 and v4 substrate now **ship public in v0.1.0
   labeled alpha / WIP** per the D-REL-1 flip. Not on the supported
   contract — see `SUPPORTED.md`.
2. Comprehensive `.dag` comment-stripping pass — load-bearing markers
   (`🟡 dissolve-on-arrival`, `🟢`/`🔴` coproduct tags, `// Anchor:`,
   dissolve-target session-slug attribution, `adhoc-<UUID>` work-item refs)
   are NOT cleanup targets.
3. Inverted-sync tooling (private → public PR flow); v0.2.0+ concern.
4. External community surface (Issues/Discussions wiring on `daglang`).
5. ~~Removal of `src/v3/` from the internal workspace (stripped from
   snapshot only).~~ **REVISED 2026-05-30:** `src/v3` now ships public
   in v0.1.0 labeled alpha / WIP per D-REL-1.
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
    not a tag gate (flavor (iv) drops Wave-2 acceptance gating);
    predicate-closure continues as MATURATION work for v0.1.1+. The
    maintainer-facing tracker is
    `docs/planning/v4-done-predicate-tracker-2026-05-30.md`; do not
    conflate gunbc `main` maturity with the v0.1.0 supported contract
    (Rust + Python + Go only). Cross-check (per `nimble-crane-490`):
    0/6 predicates PROVEN, 5 YELLOW, 1 GRAY (P6) — documented honestly
    in `SUPPORTED.md`'s v4-alpha section, not used as a strip trigger.

## D-REL decisions

DECIDED items are the working default. PENDING items carry the reviewer
recommendation as the default until the project maintainer rules otherwise.

| ID | Topic | Reviewer recommendation | Status |
|----|-------|-------------------------|--------|
| D-REL-1 | v3 + v4 substrate in public v0.1.0 | **SHIP PUBLIC, labeled alpha / WIP.** Honest-state release: no Wave-2 predicate-closure gating; full-tree ~7,951 v4 rustc errors documented (in `SUPPORTED.md` and `docs/v4-status.md`). Bar is "compilable bootstrap". Fail-closed: if v4 does not compile at tag time, the README + `SUPPORTED.md` label flips alpha → pre-alpha / experimental — no scope strip, no tag delay, just truthful labeling. v3 carries the same posture. | **REVISED 2026-05-30 (post-PM-flip):** supersedes the prior "strip src/v4" ruling and the older `RELEASE_TODO.md` §6 housecleaning legacy (which is now overridden). `scripts/publish-snapshot.sh` `STRIP_PATHS` no longer strips `src/v3` or `src/v4`. Per-surface alpha/PROVEN/GAP detail authored by `sharp-otter-407` in `docs/release/v0.1.0-v4-ship-disposition.md` (in flight, separate PR); `SUPPORTED.md` pulls from that supplement. |
| D-REL-2 | Binary distribution scope | **Advertised target = passed dry-run. No dry-run = not advertised. Source build is acceptable if binaries are flaky.** | **CONFIRMED 2026-05-30** (project maintainer). |
| D-REL-3a | Day-one daglang subset (source) | Small example-backed `.dag` subset anchored to `weather.dag` + `interp_test.dag` and the `dsl/std` vocabulary those examples exercise. Anything outside this subset is unsupported and must fail closed. | **CONFIRMED 2026-05-30.** Exact list enumerated in `docs/SUPPORTED.md` (downstream). |
| D-REL-3b | Day-one target/artifact matrix | **v0.1.0 SUPPORTED targets are Rust + Python + Go** (v2-emit). Rust passes `rustc` / `cargo check` on shipped examples today. **Python and Go honest limits:** small-smoke verified; non-trivial inputs (incl. the weather hero) **fail `py_compile`** (match-as-expression + TCO temp-decl surface bugs) and **fail `go build`** (package/module layout + `:=` scope issues) respectively. Fixes in flight, expected **v0.1.1** (Python ~2–3 working days, Go ~1–2 working days). **TypeScript is v4-alpha only**, not v0.1.0. C++/LLVM/etc. are not public v0.1.0 support. | **CONFIRMED + RECONCILED 2026-05-30** (maintainer ruling + ZERO OVERCLAIM follow-on + smart-stag-871 failure-class clarification). Failure classes are **specific surface bugs** (not pure TCO — most TCO is single-authority); disclaimer is **committed for the v0.1.0 tag tomorrow**; relaxation lifts to the v0.1.1 narrative as the named fixes land. Per-surface support level declared explicitly in `SUPPORTED.md`. |
| D-REL-4 | Public docs list | **Ship only user docs: `README`, `LICENSE`, `CHANGELOG`, `docs/GETTING_STARTED.md`, `docs/LANGUAGE.md` (or `SYNTAX.md`), `docs/CLI.md`, `docs/EXAMPLES.md`, `docs/SUPPORTED.md`, `docs/CONTRIBUTING.md` (only if public PRs are wanted); strip all other docs.** | **DECIDED 2026-05-30; enforcement PENDING.** The user-facing docs above do not all exist yet (downstream authoring work). `scripts/publish-snapshot.sh` `STRIP_PATHS` currently strips only the agent/process subtrees (`docs/briefs`, `docs/debt`, etc.); v3/v4 are no longer stripped (per the D-REL-1 (iv) flip). Root `THESIS`/`INVARIANTS`/`MODELING`/`CODING`/`TESTING` and the large `docs/thesis/`, `docs/invariants/`, `docs/planning/`, `docs/design-*` trees are also not yet stripped. A follow-up pass before tag must (a) land the user docs and (b) extend `STRIP_PATHS` to remove everything outside the D-REL-4 keep list. Gate B and Gate D catch the gap if this slips. |
| D-REL-5 | Release before v4 confidence | **YES**, under flavor (iv): v3/v4 ship public labeled alpha / WIP with honest error counts; v0.1.0's *supported contract* (Rust + Python + Go) does not depend on v4 reaching predicate-closure. | **CONFIRMED 2026-05-30** (revised post-(iv)-flip). |

## Pre-tag verification gaps (open)

A release-readiness audit on 2026-05-30 (`nimble-dove-733`, routed via
PM `still-fox-289`) surfaced five gaps that the gates aspire to close
but that no evidence currently backs. Each is a hard block on tag until
resolved.

| # | Gap | Resolution path | Owner / ETA |
|---|-----|-----------------|-------------|
| V1 | ~~TypeScript surface unsubstantiated.~~ | **RESOLVED 2026-05-30:** maintainer ruled v0.1.0 = Rust + Go + Python (the three existing v2 emit paths); TypeScript moves to v4 early-support. D-REL-3b, Goals, Non-goals, framing, and Gates A/E updated accordingly. | RESOLVED. |
| V2 | **`docs/SUPPORTED.md` does not exist.** Item D names it as the single normative authority; README, website, and release notes all derive from it. | Author `docs/SUPPORTED.md` enumerating the D-REL-3a `.dag` subset, the D-REL-3b verified surfaces (Rust + Python + Go) with per-surface support level, the v3/v4 alpha section (pulling from `sharp-otter-407`'s ship-disposition supplement), the verified install/target matrix, CLI commands, OS matrix, and fail-closed guarantee. | **WORKER DISPATCH IN FLIGHT** — `snappy-bee-513` dispatching once D-REL-3b confirmed (confirmation just landed 2026-05-30). |
| V3 | **`install.sh` PR #3992 STALLED.** Open, mergeable=MERGEABLE, 0 reviews, 0 CI checks completed; no shepherd. The doc says `curl install.sh` ships only if #3992 lands and verifies before tag. | Route a shepherd to #3992, OR drop `curl install.sh` from the v0.1.0 install path and rely solely on build-from-source. Decision needed before Gate C can pass. | **SHEPHERD CHECK IN FLIGHT** — `snappy-bee-513` checking shepherd dispatch 2026-05-30. Fallback: build-from-source becomes canonical install. |
| V4 | **Weather demo end-to-end UNVERIFIED.** Gate A requires every example to run end-to-end; the hero `dsl/examples/weather/` path against `--target rust` has not been exercised against a `target/release/gunbc` built from a clean checkout. The README hero invocation uses `--target dag` rather than the verified emit path. PM's earlier attempt (clean checkout → `cargo build` → `gunbc compile --target rust` → `cargo check`) never produced a binary. | Build `gunbc` from clean checkout; run the weather example with `--target rust`; verify generated Rust passes `cargo check`; record commit SHA + run timestamp in the Evidence column on Gate A. | **WORKER DISPATCH IN FLIGHT** — fresh `weather-demo-e2e-verification` worker being dispatched by `snappy-bee-513` 2026-05-30. |
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
  projections actually land. Modeled ≠ shipped. **The D-REL-1 (iv) flip
  is about v3/v4 substrate posture only — it does not change the
  distribution-channel timeline; Homebrew/`.deb`/APT remain v0.2.0+
  regardless.**
- **Public website:** GitHub Pages from `gunb-ai/daglang`, served at
  <https://gunb.ai>. The `daglang` PR #1 (session `fierce-dove-549`) is
  ready; the visibility/Pages flip is a launch-day maintainer action.
  The website **must obey the support matrix in `SUPPORTED.md`**: no
  claim of broad language/compiler support; CTA points to supported
  examples and the verified install path only; the website states that
  v0.1.0's verified target surfaces are **Rust, Python, and Go** (the
  three v2 emit paths) and that TypeScript is **v4-alpha only, not
  v0.1.0**.

  **Marketing-surface guards (apply to the homepage, README first
  screen, and any social/landing copy):**
  - No raw maintainer-internal figures on the marketing surface — e.g.
    `~7,951 rustc errors` and similar `v4-alpha` measurements live in
    `SUPPORTED.md` / `docs/v4-status.md`, not in the hero. The hero may
    say "v4 substrate is WIP and outside the supported contract."
  - No install/build command appears on the homepage unless it is
    verified under Gate C on every advertised target. In particular,
    `make install` / `brew install` / `apt install` are absent from the
    homepage unless and until they ship per the distribution ruling.
  - Hero claims are scoped to the v0.1.0 supported contract (Rust +
    Python + Go for the documented subset). Omni-emission, lens-as-CI,
    impossible-bug-class material is allowed in clearly-labeled
    "Vision / where this goes" sections, not in the hero or in a
    support-claim register.
  - "Impossible bug" framing is scoped: only used for bug classes
    actually PROVEN on the v0.1.0 surface; otherwise phrased as "bug
    classes the lens suite is designed to catch" or "bug classes we are
    making structurally unrepresentable" (forward-looking).
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
- **Target/artifact matrix (D-REL-3b)** — **Rust, Python, and Go** as
  the v0.1.0 supported emit targets (one per existing v2 emit lens),
  per the maintainer's authoritative ruling 2026-05-30: "for v2, i
  would focus on the three it actually emits, ts can be v4 only (in
  alpha right now)". **TypeScript is v4-alpha only**, not part of the
  v2 / v0.1.0 supported contract; it lives in `SUPPORTED.md`'s v4-alpha
  section alongside the rest of v4/v3 substrate. For each surface,
  `SUPPORTED.md` declares the support level explicitly:
  - *Full compile target* — `.dag` → emitted source → external toolchain
    check passes for the documented examples (`rustc`/`cargo check`,
    `go build`/`go vet`, `python -m py_compile`).
  - *Runnable example target* — at least one example runs end-to-end
    under the emitted target (default: Rust; Go and Python where
    declared in `SUPPORTED.md`).
  - "v0.1.0 supports X" is never used without saying what "supports"
    means.
  - **Alpha / WIP (ships but NOT on the support contract per D-REL-1
    post-flip):** `src/v3` and `src/v4` substrate. Honest error count
    documented (~7,951 v4 rustc errors at the diagnosis lane).
    `sharp-otter-407` authors the per-surface alpha/PROVEN/GAP detail
    in `docs/release/v0.1.0-v4-ship-disposition.md` (separate in-flight
    PR); `SUPPORTED.md` pulls from that supplement.
  - **Out of scope (call out explicitly):** C++, LLVM, arbitrary corpus
    emit, React app generation, self-host fixed point.
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

## v4 alpha readiness — Rust × TypeScript (per `keen-heron`, 2026-05-30)

This snapshot is the authoritative input to `SUPPORTED.md`'s v4-alpha
section. It will be **updated as Wave 2 / Wave 3 work lands**; the rows
are intentionally per-axis so progress is legible turn over turn.

| Axis | Rust | TypeScript |
|------|------|------------|
| Substrate (lines / decls) | 3,359 lines, 467 model decls | 2,737 lines (TS) + 1,732 (ECMAScript), 317 `ts_` decls |
| Realization carriers | SG-2 ✓ landed; SG-1 + SG-5 in flight | None landed (Wave 3 territory) |
| Test claims | 17 (`parse` / `manual` / `language_model`) | 6 (`parse` / `MVP1` / `manual` / `anchor`) |
| Toolchain runner | R1 + R2a + R2b + R3-external against `rustc` | None — no `tsc` invocation path |
| Verification | 4 leaf-model claim files exercise `rustc` | Structural-only (parse + grammar round-trip) |
| Known emit gap | ~7,951 `rustc` errors full-tree | **Unmeasured — no `tsc`-on-emit pipeline** |

**Critical honesty caveat (must propagate to `SUPPORTED.md`):**

> "TypeScript output is not currently checked against `tsc`;
> report-and-track basis only." — `keen-heron`

Without this, users who run v4 → TypeScript expecting clean `tsc` emit
hit surprise failures we have no count for. `SUPPORTED.md` must include
this caveat verbatim in the TypeScript-alpha section.

**Path to GREEN (informational, not gating):**

- *Rust (Wave 2, in progress):* SG-1 lands → closes ~2,978 E0423 errors
  (dominant Pareto); SG-5 lands → closes collection-realization errors.
- *TypeScript (Wave 3, not dispatched):* needs (a) SG-1/2/5
  generalization to TS, (b) `LeafModelClaim M=typescript` spec + R1
  runner, (c) `tsc`-on-emit verification pipeline. Three separate work
  items, none currently in flight.

**Net for v0.1.0 alpha labeling:**

- **Rust** → "alpha with verification framework + measured gap count"
  (~7,951 errors, falling as SG-1/SG-5 land).
- **TypeScript** → "alpha substrate-only, no verification path,
  exploratory" (must say this honestly per `keen-heron`).

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
- **Rust surface (D-REL-3b):** passes `rustc` / `cargo check` (or
  `cargo run` where the example is runnable) on **shipped examples
  today** — this is the only surface verified at example-scale.
- **Python surface (D-REL-3b) — small-smoke only:** `python -m
  py_compile` on a minimal hand-curated fixture set. Non-trivial Python
  emit (incl. the weather hero) currently **fails `py_compile`** —
  failure class is **match-as-expression + TCO temp-decl surface bugs**
  (specific bugs; not pure TCO). Fixes in flight, expected v0.1.1
  (~2–3 working days). `SUPPORTED.md` lists exactly which fixtures
  qualify and declares non-trivial Python emit as not on the v0.1.0
  support contract.
- **Go surface (D-REL-3b) — small-smoke only:** `go build` on a
  minimal hand-curated fixture set. Non-trivial Go emit currently
  **fails `go build`** — failure class is **package/module layout +
  `:=` scope issues**. Fixes in flight, expected v0.1.1 (~1–2 working
  days). `SUPPORTED.md` lists the qualifying fixtures and declares the
  rest as not on the support contract.
- **Negative tests:** unsupported-feature examples fail closed with
  named diagnostics.

**Gate B — Scope hygiene.**

- Public `SUPPORTED.md` exists and is authoritative.
- Public `README` states the v0.1.0 scope on the first screen.
- **v3 + v4 substrate ships public in v0.1.0 labeled alpha / WIP**
  (D-REL-1 post-flip). It is **not on the supported contract**;
  `SUPPORTED.md` per-surface labels what is/isn't claimed.
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

**Leak rollback (export sanitation failure).** If the publish leaks
private content (any file from the strip list, any PM/session jargon,
any unintended path):

1. **Immediately flip `gunb-ai/daglang` back to PRIVATE.**
2. Delete the release and the tag if either has been published.
3. Rotate any credentials that were exposed (tokens, keys, webhook
   secrets) — even if the leak window was short.
4. Fix the strip list (`scripts/publish-snapshot.sh` `STRIP_PATHS`) or
   the root-doc source so the leaked content cannot leak again.
5. Re-run `scripts/publish-snapshot.sh` dry-run and the public-clone
   smoke test. Only re-attempt publish once **Gate D** is green again.

**v4-compile rollback (D-REL-1 fail-closed, flavor (iv) semantics).**
If `src/v4` does not compile at tag time:

1. The README + `SUPPORTED.md` flip the alpha label to
   **pre-alpha / experimental** for the v4 surface (and v3 by parallel).
2. **No scope strip** — v3/v4 still ship public.
3. **No tag delay** — the release proceeds.
4. The truthful labeling is the entire mitigation; nothing else changes.

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
- `src/v3/` — ships public in v0.1.0 labeled alpha / WIP per D-REL-1
  (post-flip). No longer stripped.
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
- v4 ship-disposition supplement (alpha/PROVEN/GAP per-surface detail):
  `docs/release/v0.1.0-v4-ship-disposition.md` (`sharp-otter-407`,
  separate in-flight PR)
- Release-note truth table for the public GH Release body:
  `docs/release/v0.1.0-release-notes.md` (PR #4005, in flight; will be
  revised to flavor (iv) framing separately)
