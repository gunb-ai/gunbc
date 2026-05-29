# v4 deferral audit — 2026-05-29

**Operator directive (2026-05-29):** exhaustive scan + classification of every
`deferred` / `scheduled-but-deferred` / `staging` / `gated` marker in the v4
corpus.  Each is classified as **NECESSARY** (a true dependency reorder — X
waits on Y because Y is structurally not done) or **UNNECESSARY** (task
splitting; X could land in this PR/dispatch but was sliced off to a later
follow-on without an upstream substrate gap).

**Audit scope:** `src/v4/**` and `docs/v4-*.md`, `docs/design-v4-*.md`.

**Ledger standing — bounded exception (per `CLAUDE.md` "Ledger standing
principle" 2026-05-19):** this document is a **point-in-time classification
snapshot** under an explicit one-shot operator directive (2026-05-29). It is
**not** a maintained parallel ledger of `🟡 gated` / `🟡 needs-more-work` /
`scheduled-but-deferred` marks. **Authority remains with the inline marks.**

* **Dissolution trigger (this doc):** delete this file once §A1–§A8
  actions are either executed (inline marks dissolved or re-bound) or
  explicitly dismissed in PR review. No follow-up audit is owed from this
  snapshot; if a future re-classification is requested, it is a new audit,
  not an update to this one.
* **Non-maintenance pledge:** classifications below are not to be patched
  forward as the underlying marks evolve. When an inline mark dissolves
  (e.g., a T-25-core consumer migrates to `Refined<Int>`), the row in this
  doc goes stale; **that is expected and fine** — the inline mark, not
  this row, was authoritative.
* **No competing disposition:** every action item cites the inline mark or
  task-line that holds source-of-truth and either (a) recommends an edit
  to that authoritative location or (b) flags a labelling mismatch on it.
  No row of this doc is intended to override or substitute for the bind /
  dissolve-on text on the mark itself.

**Method:**

1.  Enumerate every marker. Three populations:
    * **gate vocabulary** — `🟡 gated — feature:NAME — bind X — dissolve-on Y`.
      280 occurrences across the corpus collapse to a small set of distinct
      `feature:` + `consumer:` gate names (140 distinct per §0 Population A); one row per gate.
    * **needs-more-work** — 53 `🟡 needs-more-work` markers. Each carries a
      `bind T-X — dissolve-on:` rider; classified by upstream task.
    * **prose-level deferrals** in `TASKS.md`, `src/v4/DECISIONS.md`,
      `docs/v4-compilation-milestones.md`,
      `docs/design-v4-compiler-homomorphism.md`,
      `docs/v4-close-interrogation.md`. Each row is a named scheduled-but-
      deferred item.
2.  For each marker, answer: **does the named upstream substrate exist
    today?**  If not, NECESSARY (true reorder). If yes (or if the only
    blocker is intra-task scope-splitting with no upstream gap), UNNECESSARY.

Counts below are raw `grep` populations at audit time; "1 row" in the gate
table covers all repeated annotation sites of that gate.

---

## §0. Census reproducibility — canonical grep commands

The audit uses **three distinct populations**, each measured by its own
grep. Each population answers a different question; the reader needs all
three to spot-check the §1/§5/§A numbers. **Run from repo root:**

| Population | Question it answers | Canonical grep | Result on `origin/main` |
| --- | --- | --- | ---: |
**Audit pathspec (corrected 2026-05-29 per inline review to match
declared corpus scope):** all commands below use the pathspec
`-- src/v4/ docs/v4-*.md docs/design-v4-*.md` so the census matches
the audit's declared scope. Earlier draft used `-- src/v4/` only,
missing 1 gate in `docs/v4-compilation-milestones.md`
(`feature:T-7-parse-walk-realization`).

| Population | Question it answers | Canonical grep | Result on `origin/main` |
| --- | --- | --- | ---: |
| **A. Distinct gate names** (all `feature:` + `consumer:` forms) | how many *gates* exist? | `{ git grep -hoE 'feature:[a-z][a-z0-9_-]+' origin/main -- src/v4/ docs/v4-*.md docs/design-v4-*.md \| sed 's/feature://'; git grep -hoE 'feature: [a-z][a-z0-9_-]+' origin/main -- src/v4/ docs/v4-*.md docs/design-v4-*.md \| sed 's/feature: //'; git grep -hoE 'consumer:[a-z][a-z0-9_-]+' origin/main -- src/v4/ docs/v4-*.md docs/design-v4-*.md \| sed 's/consumer://'; } \| sort -u \| wc -l` | **140** |
| **B. Total `🟡 gated` annotation rows** | how many *annotation rows* sit on fields/types? | `git grep -c '🟡 gated' origin/main -- src/v4/ docs/v4-*.md docs/design-v4-*.md \| awk -F: 'BEGIN{s=0}{s+=\$NF}END{print s}'` | **280** |
| **C. Per-gate annotation rows for gate X (all forms — header + field; covers `feature:X`, `feature: X`, `consumer:X`, `consumer: X`, and field shorthand `gated: X`)** | how many annotation rows carry gate X? | `git grep -cE 'gated[ —:]+(feature:\|consumer:)? ?X' origin/main -- src/v4/ docs/v4-*.md docs/design-v4-*.md \| awk -F: '{s+=\$NF}END{print s}'` (substitute X) | e.g. `formatter-int-refinement` = 66; `config-patch-record-projection` = 12; `formatter-cross-field-constraints` = 7; `rustfmt-deprecated-alias` = 3; `lean4-option-closed-set` = 3 |
| **C′. Per-gate FIELD annotations only (excludes `feature:`/`consumer:` header rows)** — historical/informational only | how many *field* annotations carry gate X (declaration rows excluded)? | `git grep -cE 'gated: ?X\|gated consumer: ?X' origin/main -- src/v4/ docs/v4-*.md docs/design-v4-*.md \| awk -F: '{s+=\$NF}END{print s}'` | e.g. `formatter-int-refinement` = 63; `formatter-cross-field-constraints` = 3 (clang_format only — black/ktfmt/rustfmt carry only the `feature:` header for this gate) |

**§1.x and §A all use Population C** (all-forms, includes both header
declarations and field annotations) as the canonical per-gate row
count. C′ is documented for transparency about an earlier inconsistency
where §1.1 cited the C′ value (63) while §1.3 cited the C value (7);
**§1.1 now reads "66" under C** for single-authority consistency.

**Last reproduced 2026-05-29 against `origin/main` @ `df91abc2b`:**
populations A=**140**, B=**280**.

**Corrected per inline review 2026-05-29:** the original audit cited
A=97, derived from `feature:[a-z]` (no-space form only). Inline review
caught that the inline marks use **both** spaced (`feature: NAME`,
e.g. headers in formatter files like rustfmt/clang_format) and
unspaced (`feature:NAME`) forms. The unspaced grep yields 97; the
spaced grep yields 35 distinct names; overlap = 2; **true unified
distinct = 130**. Headline gates including `formatter-int-refinement`,
`formatter-cross-field-constraints`, `lean4-option-closed-set` use
the spaced form for their headers and were excluded from the original
97 census. **The 97/89/75 figures in §1, §1.9, and downstream tables
were all undercounts** and have been corrected to 130/122/95
respectively (see §1.9 for the regenerated distribution).

This does **not** change the substantive classifications — every
gate examined for NECESSARY/UNNECESSARY in §1.1–§1.8 was a
spot-checked gate whose status was verified directly. The undercount
was in the *long-tail census denominator*, not in the tabulated
classifications. §A1's dissolve-now set (now 69 sites under Population C, was 66 under mixed C/C′) is unchanged in scope
(derived from population C per-gate counts, which were correct);
§A6's intra-task slicing list is unchanged.

**Important distinction (P2 single-authority for this audit's own
surface):** §1.1 ("66 annotation rows across 5 formatter files") and §1.3 ("7 annotation
rows across 4 formatter files") use **population C** — they count
per-field shorthand annotations like `🟡 gated: formatter-int-refinement`,
not just the `feature:` header declarations. §1.9's distribution table
uses **population A** — distinct `feature:NAME` declarations only, which
is why formatter-int-refinement contributes 1 to the 140 there but 66 to
§1.1's site count. Both numbers are correct for their respective
populations.

Reviewers running an alternative grep (e.g. counting only
`🟡 gated — feature:` headers, which yields 43; or `— feature:` allowing
a space, which yields 120) will get different numbers — none of those
are the audit's canonical commands.

---

## §1. Gate vocabulary — `🟡 gated — feature:NAME`

**140 distinct gate names** appear in the corpus (corrected 2026-05-29
per inline review — see §0 for the unified spaced+unspaced grep that
yields this; the original 97 figure missed 33 spaced-form gates
including `formatter-int-refinement` itself). **§1.1–§1.8 tabulate 8
representative gates** (5 multi-site: §1.1–§1.4 + the 13-site §1.2;
3 singletons: §1.6–§1.8); **§1.9 summarizes the remaining 132
gates**, which contain both un-tabulated multi-site gates (at 2–9
sites each) **and** the bulk of the singleton tail. See §1.9 for the
full site-count distribution table.

### §1.1 `formatter-int-refinement`  (66 annotation rows across 5 formatter files; 63 field annotations + 3 `feature:` headers — corrected 2026-05-29 per inline review for consistency with Population C)

**Bind:** T-25-core (refine substrate; `Refined<Int>` carrier with predicate
≥ 0 / ≥ 1).

**Status of upstream — CORRECTED 2026-05-29 (cursor review on this PR):**
T-25-core **IS LANDED** (`src/v4/std/refinement.dag` — PR #3354). `Refined<B>`
+ `Validation<B>` + `refine` exist; `Refined<Int>` is in active use in
`extdeps/posix.dag:20`, `extdeps/formatters/ktfmt.dag:29`, and
`extdeps/formatters/lean4_format.dag:18`.
`src/v4/TASKS.md:1252` records **`T-25 [SUBSTRATE LANDED]`**.

**Classification — CORRECTED:** **UNNECESSARY.** All 66 sites can replace
their annotated `Int` fields with `Refined<Int>` today against the existing
substrate. Each per-field "must be ≥0" / "must be ≥1" annotation becomes the
`Validation<Int>` predicate. This is the largest UNNECESSARY cluster in the
audit and inverts the headline.

**Why the original audit missed this:** the §B inspection list omitted
`refinement.dag` (action §A8 below). The bind line reads correctly; the
upstream-status check was wrong.

### §1.2 `config-patch-record-projection`  (12 annotation rows under
Population C: 1 `feature:` header in `std/patch.dag` + 11 `consumer:`
tags across 9 formatter files including 3 in prettier.dag — corrected
2026-05-29; the original "13 sites" included a non-gated prose
mention in `TASKS.md:T-4.16`)

**Bind:** T-4.16 record-field map → `*ConfigPatch` + `*_layer` projection
(`src/v4/std/patch.dag:11`).

**Status of upstream:** projection not authored in `std/patch.dag`.
Consumers (each formatter's `*ConfigPatch` / `*_layer`) are presently
hand-mirrored from `*Config` and tagged
`🟡 gated consumer:config-patch-record-projection`.

**Classification:** **NECESSARY.** The projection is a substrate feature
the consumers structurally cannot dissolve without. *However*, the
projection itself sits under **T-4.16 (this dispatch)** — see §3 below.
Landing the projection in this PR would dissolve all 12 consumer sites in
one stroke.  Recommend an action item, but each consumer's deferral is
correctly classified as a wait-on-substrate.

### §1.3 `formatter-cross-field-constraints`  (7 annotation rows across
4 formatter files: clang_format ×4, black ×1, ktfmt ×1, rustfmt ×1 —
corrected 2026-05-29 per codex + inline review; original draft missed
clang_format)

**Bind:** T-4.16 follow-on. Dissolve-on: a `*ConfigValidation` witness that
checks e.g. `max_width ≥ chain_width`, `max_width ≥ array_width`, … —
purely cross-field predicate checks expressed against existing field shapes.

**Status of upstream:** Witness substrate (`std/witness.dag`) **exists
today**.  No new std/ carrier is required — the gate is "write the
validator". `T-4.16 follow-on` is intra-task slicing, not an upstream
substrate gap.

**Classification:** **UNNECESSARY.** This is task-splitting under T-4.16,
not a structural reorder. The validator can be authored against existing
Witness primitives in the same PR that defines the config.

### §1.4 `rustfmt-deprecated-alias`  (3 annotation rows · rustfmt only;
1 `feature:` header + 2 field annotations — corrected 2026-05-29 from
"2 sites" per Population C)

**Bind:** T-4.16 follow-on. Dissolve-on: precedence rule "canonical field
wins when both are set" for `merge_imports → imports_granularity` and
`fn_args_layout → fn_params_layout`.

**Status of upstream:** No upstream substrate required — precedence is a
local predicate over the same config record.

**Classification:** **UNNECESSARY.** Same task-splitting pattern as §1.3.

### §1.5 `rustfmt-ignore-path-refinement`  (1 site · rustfmt
`ignore: List<String>`)

**Bind:** T-4.16 follow-on. Dissolve-on: replace `List<String>` with
`List<Refined<String>>` validating gitignore-format Unix paths.

**Status of upstream — CORRECTED 2026-05-29:** Same correction as §1.1 —
`Refined<B>` is generic and available now (`std/refinement.dag`,
PR #3354). `Refined<String>` requires only a `Validation<String>`
gitignore-path predicate.

**Classification — CORRECTED:** **UNNECESSARY.** Can land in T-4.16's
current dispatch.

### §1.6 `rustfmt-unstable-option-validity`  (1 site · rustfmt)

**Bind:** T-4.16 follow-on. Dissolve-on: emit stage omits "Stable: No"
fields when `unstable_features=false`. The note itself flags that the
substrate gap is **`Option<T>` carriers for unstable fields** so absence is
representable. Until then, the emit-side obligation is a guard.

**Status of upstream:** the optional carrier exists as
`v4.std.collection.Optional<T>` (`src/v4/std/collection.dag:20`;
**not** a separate `std/option.dag`, which does not exist —
corrected per inline review 2026-05-29). Migration of each unstable
rustfmt field to `Optional<T>` has not been done; the substrate is
present, the migration is in-scope T-4.16 work.

**Classification:** **UNNECESSARY.** The migration is intra-task. The
deferred "emit-side guard" is a workaround for not doing the carrier
migration that T-4.16 itself was supposed to do.

### §1.7 `lean4-option-closed-set`  (3 annotation rows · lean4_format —
corrected 2026-05-29 from "1 site" per Population C; 2 `feature:`
headers + 1 field)

**Bind:** T-4.16 follow-on. Dissolve-on: replace `lake_lean_options` open
list with typed fields **once Lean4 documents a fixed set of formatter-
relevant `set_option` keys**.

**Status of upstream:** External — Lean4 upstream does not document a
closed set today.

**Classification:** **NECESSARY** (external dependency, not internal
substrate). The dissolve trigger is **not** under v4's control; the
open-list carrier is the only faithful representation while upstream is
open. Recommend re-rooting the bind from `T-4.16 follow-on` →
`upstream:lean4-fixed-option-set` so the gate's external nature is visible
(action §A2).

### §1.8 `swift-format-rules-carrier`  (1 site · swift_format)

**Bind:** T-4.16 follow-on. Dissolve-on: add `rules: Map<String, Bool>`
once `Map` is a language-level primitive without requiring a `std` import.

**Status of upstream:** Substrate gap — there is no language-level `Map`
primitive distinct from `std/collection`.

**Classification:** **NECESSARY**, but the bind is mislabelled. This is
not a T-4.16 follow-on; it is a v4-language-level substrate gap.
Recommend retagging the bind to `bind v4-lang:map-primitive`
(action §A3).

### §1.9  Long-tail gates — remaining 132 distinct gates

**Census (regenerated 2026-05-29 from unified spaced+unspaced grep
per inline review):** the original draft cited 97 distinct gates from
the no-space `feature:[a-z]` form; that undercounted by 33 because
**33 gates use spaced `feature: NAME` headers** (including the
headline `formatter-int-refinement`). True unified distribution
across all **130** distinct gate names (from §0 population A):

| Site count | # of gates | Status |
| ---: | ---: | --- |
| 13 sites | 1 | tabulated as §1.2 (`config-patch-record-projection`) |
| 9 sites  | 1 | long tail |
| 7 sites  | 1 | long tail (`canonical-b-grounding-consumer`) |
| 6 sites  | 1 | long tail (`free-monoid-entry-generic-inference`) |
| 5 sites  | 3 | long tail |
| 4 sites  | 4 | long tail |
| 3 sites  | 4 | long tail |
| 2 sites  | 20 | long tail (17 NECESSARY long-tail; 3 already tabulated in §1.3/§1.4) |
| 1 site   | 95 | long tail singletons (§1.5–§1.8 are four of these) |
| **Total** | **130** | — |

Tabulated by name in §1.1–§1.8: **8 gates** (5 multi-site, 3 singletons).
**Remaining 122 long-tail gates** are summarized below; representative
examples (multi-site first, then singleton patterns):

* `feature:canonical-b-grounding-consumer` (7 sites) — bind T-9 algebra-ref
  grounding; T-9 not landed.
* `feature:free-monoid-entry-generic-inference` (6 sites) — bind T-2/T-9.
* `feature:section-ref-identity-evidence` (5 sites) — bind not yet landed.
* `feature:python-wave2a-decimal-int-literal` (4 sites) — bind T-4 wave-2a.
* `feature:model-core-law-expression-carrier` (4 sites) — bind T-2/T-9.
* `feature:testclaim-coproduct-reflection` (3 sites) — bind T-19 follow-up;
  matches the T-19 Phase-2 trigger (same authority as
  `t19-claim-anchor-split` below).
* `feature:t11-grammar-from-token-row` (2 sites) — bind T-11; not landed.
* `feature:network-validated-components` (2 sites) — bind T-26 boundary
  carriers; not landed.
* `feature:t19-claim-anchor-split` (3 sites) — bind T-19. **CORRECTED
  2026-05-29:** T-19 itself is closed but `src/v4/TASKS.md:1042` records an
  active **T-19 Phase-2** follow-up which is the actual dissolve trigger
  ("when T-19 Phase-2 defines separate corpus types for generated vs manual
  TestClaim rows … the union dissolves"); `std/verification.dag:196` still
  carries the matching `RULING-1: needs-more-work` mark. The original audit
  draft mis-read this as stale. **The bind is correctly active.**

The remaining un-listed long-tail gates (≈75 singletons + ≈10 doubles)
each bind to a named upstream task (T-3, T-21, T-22, T-34, T-4-wave-2b,
…) whose substrate is not yet present.

**Long-tail classification:** **NECESSARY** across the long tail
(`t19-claim-anchor-split` included — corrected per above), **with two
additional UNNECESSARY exceptions surfaced by inline review 2026-05-29
(@briansrls on PR #3880):**

* `feature:rustfmt-macro-name-refinement`
  (`extdeps/formatters/rustfmt.dag:136`) — bind `T-25-core`, landed.
  `Refined<String>` substrate exists; predicate is "valid Rust macro
  ident, not `*`". Belongs in the dissolve-now set.
* `feature:rustfmt-version-string-refinement`
  (`extdeps/formatters/rustfmt.dag:146`) — bind `T-25-core`, landed.
  `Refined<String>` substrate exists; predicate is "published semver
  version string". Belongs in the dissolve-now set.

Both are caught by the same §1.1 / §1.5 substrate-status correction
(T-25-core IS landed via PR #3354); the original long-tail sample
missed them because the §B inspection list omitted `refinement.dag`
(action §A8). They are folded into action **§A1** (rename to "DISSOLVE
all landed-T-25-core formatter refinement gates"): now 66 +
1 (`rustfmt-ignore-path-refinement`) + 1 (`rustfmt-macro-name-refinement`)
+ 1 (`rustfmt-version-string-refinement`) = **69 sites** dissolvable
against the landed `std/refinement.dag` substrate.

---

## §2. `🟡 needs-more-work` markers (53 sites)

Cluster breakdown:

| Cluster | Sites | Bind | Upstream landed? | Classification |
| --- | --- | --- | --- | --- |
| `lens/affected_set.dag` carrier scaffolds | 14 | T-21 | T-21 in progress | NECESSARY |
| `lens/testgen.dag` RULING-1 / multi-arm scheduling | 17 | T-19 LBE / T-22 eval | T-19 DONE but ruling open; T-22 not landed | MIXED (see §A5) |
| `extdeps/runtimes/v4_evaluator.dag` T-22 semantics | 1 | T-22 | not landed | NECESSARY |
| `extdeps/languages/*.dag` algebra/scalar inhabitance | 8 | T-2 / T-4 waves | wave-2b open | NECESSARY |
| `extdeps/formats/spice.dag` exact-real / voltage carriers | 2 | T-3 exact-real / T-4 wave-2 | not landed | NECESSARY |
| `extdeps/formats/sql.dag` primitive-bundle catalog derivation | 1 | substrate coproduct-variant enumeration | not landed | NECESSARY |
| `extdeps/languages/go.dag` complex-algebra / consumer | 2 | T-11 / T-4-wave-2b | not landed | NECESSARY |
| `extdeps/languages/cpp.dag` per-spelling target-profile | 1 | testcase-driven; row generator open | NECESSARY |
| Other (`lens/edit_locus.dag`, …) | 7 | Q-Regex-Primitive etc. | not landed | NECESSARY |

**Verdict:** every `needs-more-work` site is NECESSARY except the
`testgen.dag` RULING-1 cluster (§A5) where the bind reads T-19 (DONE) but
the unresolved ruling is **what `LBE = L6 form × language` should yield as
a coproduct shape** — a live design decision, not a closed substrate.
Recommend re-tagging those 17 sites with `RULING-1` (the open design
question) rather than T-19 to remove the false-closed signal (action §A5).

---

## §3. Prose deferrals

### §3.1 `TASKS.md`

| Line | Item | Bind | Classification |
| ---: | --- | --- | --- |
| 163  | **T-4.15 protocols substrate** "scheduled-but-deferred"; activates with omni-stack glue per P4 | external scope ratchet | **NECESSARY** — P4 explicitly out-of-scope for initial single-target compiler. |
| 238  | T-4.15 file authoring deferred (same) | same | **NECESSARY** (same item). |
| 66   | "Coq is the deferred second-prover probe" | T-15 R3 close design | **NECESSARY** — second prover not in critical path. |
| 727  | `EffectClassification` B3 signature-deferred | T-22 / T-23 | **NECESSARY** — depends on lens eval semantics. |
| 741  | refinement_nonempty_list etc. execution deferred to T-22 | T-22 | **NECESSARY**. |
| 907  | "de-deferred per the …" (historical) | n/a — already de-deferred | informational. |
| 1500 | `EffectSignature`, `ResourceAccess` "currently deferred by Q10" | Q10 (see §3.3) | **NECESSARY**. |
| 1522 | "deferred primitive/control paths; richer interpretation is T-22-owned" | T-22 | **NECESSARY**. |
| 1728 | Duplicate-name detection deferred to `compile_with_batch` admission | structural — admission is the rejection surface | **NECESSARY** (deliberate single-rejection-surface design, not a slip). |

### §3.2 `docs/v4-compilation-milestones.md`

| Line | Item | Bind | Classification |
| ---: | --- | --- | --- |
| 225–246 | M0/M1: `v4_evaluator` nontrivial hooks deferred to Wave 2 (T-34) | T-22 + T-34 runtime carrier | **NECESSARY** — eval semantics need the runtime substrate. |

### §3.3 `docs/design-v4-compiler-homomorphism.md`

| Line | Item | Bind | Classification |
| ---: | --- | --- | --- |
| 598  | RegisterScalarKind "deferred to a check" | T-9 inference fold | **NECESSARY**. |
| 631  | `Stage0Contract` / `CorePackage` / `CorePackageSchema` "deferred to a future dispatch" (bootstrap.dag on hold) | bootstrap.dag dispatch hold | **NECESSARY** — the dispatch is held by ratified operator decision; structurally not the regeneration-substrate cluster's scope. |
| 670  | grounded primitive bodies deferred | T-9 / IEEE primitives | **NECESSARY**. |
| 1314, 1327 | `extdeps/protocols/` deferred until glue is in scope | same as T-4.15 above | **NECESSARY** (single fact, restated). |
| 1422 | **Open Q10** — partiality and effects in ModelCore (deferred-with-trigger) | trigger = first effectful primitive lowering | **NECESSARY** — the design question is genuinely premature without a triggering use site. |
| 1447 | **Open Q13** — language versions / dialects (deferred-with-trigger) | trigger = first dialect-divergent target | **NECESSARY** — same shape. |

### §3.4 `docs/v4-close-interrogation.md`

| Line | Item | Disposition | Classification |
| ---: | --- | --- | --- |
| 12, 1122, 1329 | All §1–§12 `R4-DEFERRED` items **superseded by V4-IN-SCOPE** | historical migration | n/a (already de-deferred). |
| 153  | Tier 2 escape-hatch probe (R4-deferred bound) | exploratory | **NECESSARY** — probe scheduling. |
| 408  | R3 gate #103 question — CI-integration deferred to slice cascade? | open design | **NECESSARY** (decision pending). |
| 627, 758 | Probes deferred to post-R4-canvas-ratification | scheduling | **NECESSARY**. |
| 1084 | Lint: `docs/audit/*.md` authored without §1.8/§10 row OR `R4-deferred` disposition | discipline | informational (not a deferral; a lint surface). |

### §3.5 `docs/v4-dag-rationale.md`

Same two items as §3.3 line 598 & 670 (cross-referenced); already covered.

### §3.6 `src/v4/DECISIONS.md`

Single `deferred` hit — historical context, no live item.

---

## §4. `staging` and other markers

Aggregate `staging` hits are dominated by domain vocabulary inside
formatter configs (e.g. clang-format's *staging-area* lines, rustfmt's
*staged* / *unstable* terminology). **These are noun usage of the upstream
tools' jargon, not deferral markers** for v4 substrate. None misclassified.

Sampled: `extdeps/formatters/clang_format.dag` — **regenerated from
`grep -oE '🟡 gated[: ]+[a-z-]+'` 2026-05-29:**

| Gate | Annotation rows in clang_format.dag | Routes to |
| --- | ---: | --- |
| `formatter-int-refinement` | 35 | §1.1 (UNNECESSARY) |
| `formatter-cross-field-constraints` | 3 | §1.3 (UNNECESSARY) |
| `consumer:config-patch-record-projection` (consumer tag) | 1 | §1.2 (NECESSARY) |

Plus header `feature:` declarations for each (not field annotations).
**Earlier claim that "all clang_format gated rows trace to §1.1" was
wrong** — corrected per inline review 2026-05-29. The §4 conclusion
holds: **zero v4 deferral inside a "staging" noun**; every gated row
in clang_format routes to a tabulated §1 entry. The original sentence
under-counted the route by omitting §1.2 and §1.3.

---

## §5. Summary — CORRECTED 2026-05-29 (post-cursor-review)

| Population | Sites | NECESSARY | UNNECESSARY |
| --- | ---: | ---: | ---: |
| `🟡 gated — feature:*`+`consumer:*` distinct gates (Population A) | 140 (full scope incl. docs/) | 133 | **7** (§1.1 formatter-int-refinement, §1.3 cross-field, §1.4 deprecated-alias, §1.5 ignore-path-refinement, §1.6 unstable-option-validity, §1.9 rustfmt-macro-name-refinement, §1.9 rustfmt-version-string-refinement) |
| `🟡 gated` annotation occurrences (Population C, all forms, full scope) | 280 | ~200 | **~80** (§1.1 = 66 + §1.3 = 7 + §1.4 = 3 + §1.5 = 1 + §1.6 = 1 + 2 long-tail T-25-core binds = 80; all derived from §0 Population C — single authority) |
| `🟡 needs-more-work` | 53 | 36 | 17 (§A5 testgen RULING-1 mis-tag, design-open not substrate-blocked) |
| Prose deferrals (TASKS / docs) | ~25 | ~25 | 0 |

**Headline — REVISED:** the v4 deferral ledger has **one large unnecessary
cluster** — `formatter-int-refinement` (66 sites across 5 formatter
files under Population C; the broader landed-T-25-core dissolve-now
set is 69 sites — see §A1)
should have dissolved when T-25-core (`std/refinement.dag`) landed in
PR #3354. Prose deferrals and the long tail of feature gates remain
honest. Combined with the smaller T-4.16 follow-on gates (§1.3, §1.4, §1.6 —
§1.5 already folded into §A1's 69-site set), there is a clear T-4.16
close-out opportunity to dissolve **~79 yellow marks** against
substrate that already exists: 69 (§A1 landed-T-25-core set under
Population C) + 7 (§1.3 cross-field) + 3 (§1.4 deprecated-alias under
Population C) + 1 (§1.6 unstable-option-validity) = **80**. Matches §5's
~80 row.

The misclassifications cluster in **two places**:

1.  **One stale-substrate cluster (§1.1)** — `formatter-int-refinement`
    still tagged "wait on T-25-core" though T-25-core landed PR #3354.
2.  **Four T-4.16 "follow-on" gates** that are intra-task slicing, not
    upstream reorder (§1.3, §1.4, §1.5, §1.6). All four can land in
    T-4.16's current dispatch against substrate that exists today
    (Witness, `v4.std.collection.Optional<T>`, Refined<B>).

One labelling defect: **§1.8** (`swift-format-rules-carrier`) — bind
points at "T-4.16 follow-on" but the real upstream is a v4-language Map
primitive (action §A3). The `testgen.dag` RULING-1 cluster (§A5) is also a
labelling defect: work is open, bind names a closed task.

---

## §A. Action items

* **§A1 — DISSOLVE all landed-T-25-core formatter refinement gates
  (69 sites total under Population C)** against the landed
  `std/refinement.dag` substrate. Breakdown: 66 × `formatter-int-refinement`
  rows (63 field annotations + 3 `feature:` headers) +
  1 × `rustfmt-ignore-path-refinement` + 1 × `rustfmt-macro-name-refinement`
  + 1 × `rustfmt-version-string-refinement`. Replace each annotated
  `Int` / `String` field with `Refined<Int>` / `Refined<String>` plus the
  per-field `Validation<*>` predicate. Reference patterns:
  `extdeps/posix.dag` ProcessId/ExitCode wrappers and
  `extdeps/formatters/ktfmt.dag:29`. **This is the largest single
  dissolution available** in the v4 corpus today.
* **§A2 — re-bind `lean4-option-closed-set`** from `T-4.16 follow-on` →
  `upstream:lean4-fixed-option-set` to surface the external dependency.
  Edit in `lean4_format.dag`.
* **§A3 — re-bind `swift-format-rules-carrier`** from `T-4.16 follow-on` →
  `v4-lang:map-primitive`. Edit in `swift_format.dag`.
* **§A4 — (RETRACTED 2026-05-29).** Original draft proposed dissolving
  `feature:t19-claim-anchor-split` consumers. **Wrong** — `TASKS.md:1042`
  records active T-19 Phase-2 follow-up that is the actual dissolve
  trigger; `std/verification.dag:196` still carries the matching
  `RULING-1: needs-more-work`. The bind is correctly active. No action.
* **§A5 — re-bind `lens/testgen.dag` RULING-1 cluster** (17 sites) from
  T-19 to `RULING-1` (the open design ruling), so the open status is
  honest. Sweep edit.
* **§A6 — UNNECESSARY: collapse four intra-task gates into T-4.16's
  current dispatch:**
  * `formatter-cross-field-constraints` (7 annotation rows across 4
    formatter files — rustfmt, ktfmt, black, clang_format) — author
    `*ConfigValidation` witnesses for rustfmt / ktfmt / black against
    existing Witness substrate.
  * `rustfmt-deprecated-alias` (3 annotation rows under Population C: 1
    `feature:` header + 2 field annotations) — author the canonical-wins
    precedence predicate over `RustfmtConfig`.
  * `rustfmt-unstable-option-validity` (1 site) — migrate unstable
    rustfmt fields to `Optional<T>` (substrate present as
    `v4.std.collection.Optional<T>` in `std/collection.dag:20`).
  * `rustfmt-ignore-path-refinement` (1 site) — replace
    `List<String>` with `List<Refined<String>>` against landed
    `std/refinement.dag` (cross-ref §A1).

  **None of these requires upstream substrate work.** Recommend folding
  into T-4.16's close conditions (or a tight follow-on PR explicitly
  tagged as task-completion, not substrate-wait).

* **§A7 — `config-patch-record-projection` is the highest-leverage
  remaining substrate dissolution** (12 consumer sites + 1 substrate
  site). Although correctly classified NECESSARY today, landing the
  projection in `std/patch.dag` immediately dissolves all 12 formatter
  consumer hand-mirrors. Recommend prioritising the projection inside
  T-4.16.

* **§A8 — audit methodology fix:** the §B inspection list omitted
  `src/v4/std/refinement.dag`, which caused the §1.1 / §1.5 NECESSARY
  miscall. Future audit passes that touch `Refined<*>` claims must check
  `refinement.dag` directly (or use `TASKS.md` `[SUBSTRATE LANDED]`
  markers as the authoritative status surface).

---

## §B. Methodology notes

* Counts are raw `grep` populations; per-annotation sites of a single gate
  name collapse to one classification row.
* "Substrate present?" was checked by direct file inspection of
  `src/v4/std/` (`patch.dag`, `collection.dag` for `Optional<T>`,
  `witness.dag`) and by `git log`
  status of T-tasks named in bind lines. **Original draft omitted
  `refinement.dag` from this list** — fixed in §A8; the §1.1 and §1.5
  rows were corrected after cursor review surfaced the gap.
* Where a bind line is misleading (§1.5, §1.8, §A5), the misclassification
  was charged against *labelling*, not against the deferral itself; the
  underlying status (NECESSARY / UNNECESSARY) is reported per the actual
  substrate state.
* `staging` as a marker word was disambiguated from formatter-tool
  jargon by inspection; no v4 substrate uses "staging" as a deferral term
  (the project vocabulary uses `🟡 gated`, `🟡 needs-more-work`,
  `scheduled-but-deferred`, or `deferred-with-trigger`).
