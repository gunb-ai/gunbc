# SG-7 §10 Worksheet — CI byte-offset recursion dissolves into `v4.std.node` `byte_offset_cache_key`

> **Status:** WORKSHEET — Modeling DFS Manager (proud-pike-680) §8 sign-off REQUIRED before any implementation worker may be dispatched.
> **Date:** 2026-05-30
> **Author:** wise-badger-838 (worker under proud-pike-680)
> **Class:** SG-7 (single-authority candidate) — substrate-extension scope, not spot-fix.
> **Blocks:** PR #3974 follow-on (interim CI relief is transport-wiring only; this is the structural dissolution it gestures at); T-22 / `feature:T22-EVAL-CACHE-HASHES` close.
> **Mechanical rule (PR #3938 §11.1 / sibling worksheet §"Mechanical dispatch rule"):** No SG-7 implementation worker may be dispatched until this worksheet identifies the single-authority fact to add or consume and Modeling DFS Manager checks §8.

---

## §0 One-sentence hypothesis

`v4.std.node` `byte_offset_cache_key` (typed `ByteOffsetCacheKey` coproduct with eligible-digest / ineligible-witness variants, O(log |i|) base-256 fold, P4 high-band peel) IS the single authority for byte-offset cache identity; `src/v4/workflow/ci.dag:721` `ci_int_offset_authority_projection_node` is a **parallel recursive Node-tree author** of the same semantics and dissolves by consuming a `Node` projection of `ByteOffsetCacheKey` from `v4.std.node` — **no new vocabulary**, only one missing projection on the existing authority.

---

## §1 §10.0 worksheet (schema-class shape, per `v4-correctness-ladder` §10.0 + sibling `v4-ci-schema-worksheet` adaptation)

```text
Schema class:           SG-7 (CI byte-offset projection — single-authority candidate)
Representative failure: src/v4/workflow/ci.dag:721 `ci_int_offset_authority_projection_node`
                        recursively re-authors a base-256 limb tree (via `ci_byte_limb_projection_node`,
                        ci.dag:704) whose semantics are already the operator-ratified
                        `byte_offset_cache_key_from_authority` (std/node.dag:849) consumed by
                        `byte_offset_cache_key` (std/node.dag:1027). The recursion is per-character
                        (called from `ci_char_projection_node` :766 and consumed by
                        `ci_char_cache_digest` :949) and inflates T-22 eval-cache projection cost on
                        every CI string the pipeline touches — the empirical motivation behind PR #3974
                        interim relief (operator 2026-05-30: "CI is taking 30 minutes + right now").

Immediate local patch (FORBIDDEN — would be the spot fix):
  - Inline a `cache_key: Hash` field on `ci_byte_limb_projection_node` callers.
  - Add a `ci_int_offset_cache_digest(i: Int) -> Hash` shim in ci.dag that hashes the recursive Node
    once and memoizes — but the authoring node tree still exists, so still a parallel author.
  - Lift the recursion into a private helper in ci.dag while keeping the recursion shape — same P2 +
    Practice 11 (parallel payload) violation, just renamed.
  - "Just stop emitting offsets on docs-only PRs" — what PR #3974 lane B does. Legitimate transport
    wiring, but does NOT dissolve the authority parallelism; the recursive tree still runs every time
    `v4` is affected (i.e., the common case).

Why forbidden:
  - **P2 single-authority:** two definitions of "what is the structural identity of a byte offset?"
    on the same row of the model — `byte_offset_cache_key` (std/node) and
    `ci_int_offset_authority_projection_node` (ci.dag). When they drift (e.g., one widens the
    eligibility ceiling, the other doesn't), cache rows from one author silently miscompare with the
    other. INVARIANTS P2.
  - **Practice 10 (Merkle catamorphism) / Practice 11 (parallel payload):** ci.dag is authoring a
    payload tree whose canonical projection already lives in std/node; the ci.dag tree is the
    parallel payload. Per `v4-ci-schema-worksheet-2026-05-30` §1.5, cache identity is `content_hash`
    of a projection of the authoritative subgraph, never a hand-authored shadow.
  - **MODELING.md M9 (DFS the concept DAG):** "byte offset cache identity" already has a concept
    home (`v4.std.node`); a second author in `v4.workflow.ci` creates a divergent path through the
    DAG for the same concept.
  - **INVARIANTS P1 (closed system):** any divergence between the two authors is recoverable to a
    missing structural fact, namely a `Node` projection of `ByteOffsetCacheKey` exposed from
    `v4.std.node`. Spot-fixing in ci.dag would entrench the parallel author and leave the missing
    fact unmodeled.

DFS path (per MODELING.md M9; concept-home DFS from `dsl/std/` outward):
  std/ authority (deepest):
    - dsl/std/types.dag        — Hash, Int, Char, atom_identity_hash, combine_hash
    - dsl/std/patterns.dag     — Merkle catamorphism / cache-as-projection canon (Practice 10)
  v4 substrate authority (the single-authority candidate):
    - src/v4/std/node.dag
        :473-486  ByteOffsetCacheKey coproduct (Eligible { digest } | Ineligible { boundary_tag,
                  witness }) + canonical_tag_byte_offset_cache_key_{eligible,ineligible}
        :600-625  byte_offset_overflow_quotient_high_band_terminal_digest (the base-256 limb fold)
        :626-655  P4 byte_offset_overflow_quotient_high_band_digest (Hash-only high-band peel;
                  "never re-enters byte_offset_cache_key_from_authority / witness_digest" — the
                  termination discipline that ci.dag's recursion currently lacks)
        :676-700  byte_offset_cache_digest_ineligible_witness_digest / _ineligible_hash
                  (ineligible-tagged digest for unrepresentable / out-of-ceiling boundaries)
        :712-845  byte_offset_cache_digest_overflow_*  (peel-bounded authority — bounded recursion;
                  the structural counter-example to ci.dag's unbounded /256 recursion)
        :849-906  byte_offset_cache_key_from_authority  (the eligible/ineligible projection)
        :1027     byte_offset_cache_key(i: Int) -> ByteOffsetCacheKey  (public API)
        :1033     byte_offset_cache_key_fingerprint(key: ByteOffsetCacheKey) -> Hash
                  (operator-eligible-vs-ineligible tagged fingerprint — the canonical Hash projection)
  v4 consumer authority (already partially aligned):
    - src/v4/compiler/05_eval.dag:76-77 imports ByteOffsetCacheKey + byte_offset_cache_key (the
      T-22 evaluator path consumes the canonical authority; CI's recursion is the holdout)
    - src/v4/compiler/05_eval.dag:589-618 ByteRange extent uses base-256 cache key, NOT Peano —
      explicitly contrasts with the ci.dag holdout per the comment "O(log |i|) base-256
      `byte_offset_cache_key` (not Peano on |i|); ineligible offsets fail-closed at the cache
      boundary."
  v4 holdout (the dissolution target):
    - src/v4/workflow/ci.dag
        :57       imports byte_offset_cache_key (already imported — but only consumed for Char
                  cache digest at :952, NOT for the offset Node projection at :721)
        :703      🟡 derived-operation comment ALREADY NAMES the dissolution:
                  "dissolve-on: std/node canonical `byte_offset_cache_key` Node projection
                   replaces this tree. Live consumer: `content_hash(ci_pipeline_evaluator_input_node)`
                   (not `ci_char_cache_digest`)."  ← operator-pre-ratified dissolution path
        :704-720  ci_byte_limb_projection_node  (the base-256 limb tree)
        :721-754  ci_int_offset_authority_projection_node  (the recursive offset tree — THIS is
                  the parallel author)
        :766      ci_char_projection_node consumes ci_int_offset_authority_projection_node
                  (the call site that re-enters the recursion per character)
        :931-944  ci_pipeline_evaluator_input_node  (the live consumer named in the :703 comment)
        :949-964  ci_char_cache_digest  (already correctly consumes byte_offset_cache_key for Char;
                  the structural counter-example WITHIN ci.dag of the right pattern)

  extdeps:    (none — closed-system substrate work)
  compiler stage: workflow emission + T-22 eval projection (NOT 04_infer / 05_emit)
  scaffold notes:
    - The dissolution is ASYMMETRIC: `byte_offset_cache_key` returns a TYPED `ByteOffsetCacheKey`
      (Eligible | Ineligible), not a `Node`. ci.dag needs a `Node` projection. So the single
      authority is correct, but std/node does not currently expose `byte_offset_cache_key`'s
      *Node projection* — only its *Hash fingerprint* (:1033). The missing single-authority fact
      is exactly: a `byte_offset_cache_key_projection_node(key: ByteOffsetCacheKey) -> Node`
      (or equivalent name per manager §8) on `v4.std.node`, such that
      content_hash(byte_offset_cache_key_projection_node(byte_offset_cache_key(i))) is the
      canonical cache identity that ci.dag MUST consume in place of authoring its own tree.
    - Once that projection exists, `ci_int_offset_authority_projection_node` reduces to a one-line
      delegation; `ci_byte_limb_projection_node` becomes unreferenced and deletes (or also
      delegates to a `byte_limb_projection_node` on std/node, if the limb-level Node has its own
      consumers — manager call).
    - `ci_char_projection_node` at :766 then consumes the std/node projection for the offset arm.
    - 🟡 marker at :703 stays until dissolution lands; PR title should reference SG-7 + the dissolve-on tag.

Deepest unsound boundary:
  `v4.std.node` exposes the operational typed authority for byte-offset cache identity
  (`ByteOffsetCacheKey` + `byte_offset_cache_key_from_authority` + `byte_offset_cache_key_fingerprint`)
  but does NOT expose a `Node`-shaped projection of that authority. `v4.workflow.ci` consequently
  authors a parallel recursive Node tree (`ci_int_offset_authority_projection_node`) to fill the
  gap, recovering the missing structural fact informally and unboundedly (no P4-equivalent
  high-band-peel termination discipline; raw `i / 256` recursion with no limbs_remaining counter).

Systemic fix (single-authority addition, in order):
  1. (Modeling DFS §8) Approve the Node-projection addition on `v4.std.node` — name, signature,
     and whether the projection lives next to `byte_offset_cache_key_fingerprint` (:1033) or
     adjacent to `byte_offset_cache_key_from_authority` (:849). Document the relationship between
     the new Node projection and the existing Hash fingerprint:
         content_hash(byte_offset_cache_key_projection_node(k)) ≡ byte_offset_cache_key_fingerprint(k)
     (proof obligation on the worker via TestClaim — see §3 falsification probe 1).
  2. Land the Node projection in `v4.std.node` (no consumers yet — substrate-first, per
     model-before-implement).
  3. Replace ci.dag:721 `ci_int_offset_authority_projection_node` body with delegation to the
     new projection (one-line body). Delete `ci_byte_limb_projection_node` (:704) IF it has no
     remaining ci.dag consumers; otherwise also delegate.
  4. Remove the 🟡 dissolve-on comment at ci.dag:703.
  5. Update T-22 cache-hash close-status references in `src/v4/TASKS.md` (T-22 §"T22-EVAL-CACHE-HASHES";
     `feature:T22-EVAL-CACHE-HASHES`) to cite the dissolution.

Non-goals (out of scope for the SG-7 PR):
  - Widening / narrowing the byte-offset ceiling (255 / 256-quotient boundary in :735–:751).
  - Changing the eligible/ineligible coproduct shape (operator-ratified at std/node.dag:473).
  - Touching `ci_char_cache_digest` (:949) — already correctly consumes the canonical authority.
  - Replacing `ci_pipeline_evaluator_input_node` (:931) — it is the live consumer per the :703
    comment; only its dependency chain shortens.
  - The CI Phase 1.4/1.5/2/2.5 overhaul (per `v4-ci-overhaul-2026-05-30.md`) — this SG-7 is
    orthogonal: it dissolves a holdout substrate parallelism that exists regardless of the overhaul,
    and unblocks T-22 close independent of `CiUpsertStep<T>` landing.
  - Editing `.github/workflows/ci.yml` (T-24 Phase 2 territory; forbidden as authority per
    INVARIANTS P2).
  - Any change to PR #3974's interim relief lanes (B-fmt-skip is transport wiring; this is
    structural).

Falsification probes (acceptance — see §3 for full TestClaim shape):
  1. (REQUIRED) `content_hash(byte_offset_cache_key_projection_node(byte_offset_cache_key(i))) ==
                 byte_offset_cache_key_fingerprint(byte_offset_cache_key(i))` for the existing
     `test_claim_cache_digest_sensitivity` corpus (:354–:413).
  2. (REQUIRED) `content_hash(ci_int_offset_authority_projection_node(i))` BEFORE the dissolution
     equals `content_hash(byte_offset_cache_key_projection_node(byte_offset_cache_key(i)))` AFTER —
     i.e., the dissolution is digest-preserving for the live `ci_pipeline_evaluator_input_node`
     consumer (no cache invalidation across the dissolution PR).
  3. (REQUIRED) Grep gate: post-dissolution, NO recursive `if i == 0 / else if i <= 255 / else
     <self-recurse>` Node-building pattern remains in `src/v4/workflow/ci.dag`.
  4. (REQUIRED) Grep gate: no new `*_cache_key` / `*_offset_*` Symbol-keyed shim introduced
     in ci.dag (would be a parallel-payload spot fix).
  5. Boundary: ineligible offsets (i > ceiling) project to a Node whose `content_hash` matches
     the ineligible-witness digest path through `byte_offset_cache_digest_ineligible_witness_digest`
     (std/node.dag:676) — exercise via the `tcc_cache_byte_offset_over_ceiling` claims
     (test_claim_cache_digest_sensitivity.dag:354).

Metric allowed only as secondary:
  - Wall-clock CI delta on PRs that touch v4 substrate (PR #3974 motivation). Report ONLY after
    probes 1–5 pass. The structural success criterion is digest equality + recursion-eliminated,
    NOT wall-clock — per `v4-ci-schema-worksheet-2026-05-30` §"Structural success criterion".
```

---

## §2 DFS concept-home map (M9)

```text
Concept                                 | Home (authoritative)                       | SG-7 action
----------------------------------------|--------------------------------------------|-----------------------------------------
Byte-offset cache identity (typed)        | v4.std.node ByteOffsetCacheKey               | Consume (no change)
Byte-offset cache Hash fingerprint        | v4.std.node byte_offset_cache_key_fingerprint  | Consume; tie via Node projection equality
Byte-offset cache Node projection         | v4.std.node (NEW — single new fact)            | ADD — exact name §8 manager call
Base-256 limb Node                          | v4.std.node (candidate; today only in ci.dag)  | Manager §8: lift or delegate
Ineligible-witness digest                   | v4.std.node byte_offset_cache_digest_ineligible_witness_digest | Consume (boundary preserved)
P4 high-band peel termination               | v4.std.node byte_offset_overflow_quotient_high_band_digest      | Replaces ci.dag's unbounded recursion
T-22 eval cache projection                  | v4.workflow.ci ci_pipeline_evaluator_input_node | Dependency tree shortens; node unchanged
Char cache digest (counter-example, ok)     | v4.workflow.ci ci_char_cache_digest          | Unchanged — already canonical
Char Node projection                        | v4.workflow.ci ci_char_projection_node        | Replace offset-arm call site (:766)
```

**New concepts — manager §8 closed questions:**

1. Name of the new projection: `byte_offset_cache_key_projection_node`? `byte_offset_cache_key_node`? `node_of_byte_offset_cache_key`? Manager call; suggest `byte_offset_cache_key_projection_node` to mirror existing `*_projection_node` cadence across `src/v4/workflow/ci.dag` + `src/v4/std/`.
2. Does `byte_limb_projection_node` lift to `v4.std.node` (alongside the cache key), or stay deleted? Recommend lift IFF any non-ci.dag consumer exists (today: none found via grep — recommend delete after dissolution).
3. Does the dissolution PR also delete the `🟡 derived-operation` comment at ci.dag:703, or leave a `# SG-7 dissolved <PR#>` breadcrumb? (No-comment per CODING.md unless the next reader would be confused; recommend delete, citing PR in commit message only.)

---

## §3 Falsification probes — TestClaim shape (no impl yet — for the implementation worker brief)

Each probe is a `TestClaim` placed under `src/v4/test/claim/std_node/` (probes 1, 5) or `src/v4/test/claim/workflow/` (probes 2–4) per concept home.

| # | TestClaim | Subject under test | Pass condition |
|---|---|---|---|
| 1 | `byte_offset_cache_key_projection_node_matches_fingerprint` | `byte_offset_cache_key_projection_node` (new) | `content_hash(byte_offset_cache_key_projection_node(byte_offset_cache_key(i))) == byte_offset_cache_key_fingerprint(byte_offset_cache_key(i))` over the existing `test_claim_cache_digest_sensitivity` Int corpus (:354–:413) |
| 2 | `ci_int_offset_authority_projection_dissolution_digest_preserving` | `ci_int_offset_authority_projection_node` (pre) vs new delegation (post) | Snapshot vector of digests over the same Int corpus is bit-identical pre/post; landed in the dissolution PR |
| 3 | `ci_workflow_no_recursive_byte_offset_node_author` | `src/v4/workflow/ci.dag` body | Grep gate: no `if i == 0 / else if i <= 255 / else <self-recurse>` Node-building pattern. Realised as a fixture-style claim per `v4-leaf-model-verification-2026-05-30.md` shape. |
| 4 | `ci_workflow_no_parallel_byte_offset_cache_shim` | `src/v4/workflow/ci.dag` body | Grep gate: no new `*_byte_offset_cache_*` symbol introduced in this PR beyond consumption of std/node's API |
| 5 | `byte_offset_cache_key_projection_ineligible_boundary` | new projection on `ByteOffsetCacheIneligible { boundary_tag, witness }` | `content_hash` matches `byte_offset_cache_digest_ineligible_hash` path; exercises `tcc_cache_byte_offset_over_ceiling`, `tcc_cache_byte_offset_same_quotient`, and the `pow_19..pow_37` claims at :389–:413 |

---

## §4 Spot-fix register (forbidden — grep gate for reviewers of the implementation PR)

| Pattern | Why forbidden |
|---|---|
| `fn ci_int_offset_*` reintroducing recursion on `i / 256` in ci.dag | Re-authoring the parallel tree under a new name |
| `fn ci_byte_limb_*` retained in ci.dag after dissolution | Parallel author for the limb sub-structure |
| `cache_key: Hash` or `cache_digest: Hash` payload field on any new ci.dag carrier | Practice 11 parallel payload — projection-derived only |
| A new `Symbol` table mapping offset literals to digests | Heuristic enum / string-keyed authority — forbidden per `v4-ci-schema-worksheet` §4 |
| New `BoundaryTag` enum in ci.dag | Boundary tags live on `v4.std.node` only (`canonical_tag_byte_offset_*`) |
| `ci_int_offset_authority_projection_node` exported / referenced from outside ci.dag | The function should be private (and post-dissolution, a one-line delegation; ideally inlined and deleted) |
| Adding a `byte_offset_cache_key_projection_node` variant in `v4.workflow.ci` instead of `v4.std.node` | Wrong concept home (M9 violation) |
| Touching `.github/workflows/ci.yml` in the same PR | YAML-authority dissolution — T-24 Phase 2 territory; out of SG-7 scope |
| Wall-clock CI delta cited as acceptance gate | Structural correctness is the gate; wall-clock secondary |

---

## §5 Why this is SG-class (single-authority), not spot-fix

Three structural facts:

1. **The dissolve-on marker is already ratified.** `src/v4/workflow/ci.dag:703` carries the explicit `dissolve-on: std/node canonical \`byte_offset_cache_key\` Node projection replaces this tree.` The substrate-direction is operator-pre-ratified; SG-7 is the act of *closing* that 🟡 marker, not opening new design space.

2. **The canonical authority already exists in `v4.std.node`.** `ByteOffsetCacheKey` (:473), `byte_offset_cache_key` (:1027), `byte_offset_cache_key_fingerprint` (:1033), and the P4 high-band-peel termination discipline (:626) are all landed. ci.dag is the holdout. There is no new vocabulary to coin — exactly one missing structural fact (Node projection of the existing typed authority).

3. **A counter-example to the spot-fix is already inside ci.dag.** `ci_char_cache_digest` (:949) consumes `byte_offset_cache_key` correctly via the canonical API. The offset-tree author at :721 is the asymmetric holdout — same author, same file, different discipline. The dissolution makes the file internally consistent.

If this were spot-fix territory (per `v4-ci-overhaul-2026-05-30.md` §4 B2/B3 framing), the fix would live in ci.yml `if:` conditions or a ci.dag-local memoization shim. It does not, because the parallel-author is a substrate fact, not a transport-wiring fact.

---

## §6 Dispatch shape (after manager §8 sign-off)

```text
ONE worker, ONE PR. No splits; the dissolution is digest-preserving and must land atomically
to avoid a window where two authors disagree.

MUST:
  - Add the §8-approved Node projection to v4.std.node (single new fn; no new types).
  - Replace ci.dag:721 body with delegation; delete ci_byte_limb_projection_node if unreferenced.
  - Land §3 TestClaims 1, 2, 5 (3, 4 are grep-gate fixtures, may be a follow-on per
    v4-leaf-model-verification §8 worksheet).
  - Remove the 🟡 marker at ci.dag:703.
  - Cite this worksheet + the v4-ci-overhaul §11 anchor + PR #3974 in the PR description.

MUST NOT:
  - Touch CI Phase 1.4/1.5 substrate (CiUpsertStep<T> / UpsertInputRef — different lane).
  - Edit .github/workflows/ci.yml.
  - Introduce any §4 forbidden pattern.
  - Use the worksheet as license to widen the byte-offset ceiling or restructure
    ByteOffsetCacheKey (operator-ratified at std/node:473).

Escalate (do not improvise) — single-authority bar:
  - Any std/node consumer (outside ci.dag + 05_eval) is found to depend on the recursive ci.dag
    tree's specific Node shape (would mean ci.dag was unintentional authority — escalate to
    Modeling DFS Manager + operator before dissolution).
  - The new Node projection's content_hash does NOT match byte_offset_cache_key_fingerprint
    over the existing Int corpus (means the Hash fingerprint and the proposed Node projection
    are structurally divergent — the SG-7 hypothesis fails, escalate).
  - ci.dag has a consumer of `ci_int_offset_authority_projection_node` that the dissolution
    cannot serve without reshaping the receiver (means SG-7 is not actually a one-fact dissolution
    — escalate, do not spot-fix the receiver).
```

---

## §7 Open questions for Modeling DFS Manager (§8 sign-off)

- **Q-SG7-1.** Concept-home for the new Node projection: `v4.std.node` (alongside `byte_offset_cache_key_fingerprint`)? — *Proposed: yes; same module owns the typed authority + both projections (Hash, Node).*
- **Q-SG7-2.** Name: `byte_offset_cache_key_projection_node`? — *Proposed: yes; mirrors existing `*_projection_node` cadence.*
- **Q-SG7-3.** Disposition of `ci_byte_limb_projection_node` (:704) — delete after delegation, or lift to `v4.std.node` as `byte_limb_projection_node`? — *Proposed: delete; no non-ci.dag consumer.*
- **Q-SG7-4.** Falsification probes 3 & 4 (grep gates) — land in the SG-7 PR or a `v4-leaf-model-verification` follow-on per its §8 worksheet? — *Proposed: follow-on; SG-7 PR carries the digest-preservation claim (probe 2) and the projection-vs-fingerprint claim (probe 1) only.*
- **Q-SG7-5.** Should the dissolution also update `src/v4/TASKS.md §T-22 T22-EVAL-CACHE-HASHES` close-status note to reference the SG-7 PR, or is that a separate Close/Receipt Manager call? — *Proposed: include in SG-7 PR (one-line cross-reference) since the dissolution closes the 🟡 marker that the close-status text cites.*

---

## §8 Manager approval checklist (proud-pike-680) — OPEN

- [ ] Hypothesis (§0): `v4.std.node` `byte_offset_cache_key` is the single authority; ci.dag:721 is the dissolving parallel author.
- [ ] DFS path (§1) walked from `dsl/std/` outward; no concept-home alternative surfaced.
- [ ] §4 spot-fix register adopted as reviewer grep gate.
- [ ] §3 probe 1 (projection ≡ fingerprint) confirmed as the structural acceptance gate.
- [ ] §7 Q-SG7-1..5 answered (or deferred to operator).
- [ ] Worker dispatch — **blocked until checklist closes.**

---

## §9 Non-goals (this worksheet)

- Implementation. (No `.dag` edits in the same PR as this worksheet; per the brief's `NO impl until DFS §8`.)
- Re-litigating PR #3974's interim-relief tradeoffs (lanes A/C dropped per PR #3974 body; B landed). Transport wiring; orthogonal to SG-7.
- The broader CI Phase 1.4/1.5/2/2.5 overhaul (`v4-ci-overhaul-2026-05-30.md`). SG-7 is parallel to the overhaul, not part of its critical path.
- TestClaim runner gating (T-38 territory). SG-7's TestClaims compile per the existing `test_claim_cache_digest_sensitivity` discipline; execution lands when T-38 lands.

---

## §10 Related artifacts

- `src/v4/std/node.dag:473-486, :600-700, :849-906, :1027-1051` — the single-authority surface.
- `src/v4/workflow/ci.dag:57, :703-754, :766, :931-944, :949-964` — the holdout + the in-file counter-example.
- `src/v4/compiler/05_eval.dag:76-77, :589-618` — T-22 consumer already on the canonical path.
- `src/v4/test/claim/manual/test_claim_cache_digest_sensitivity.dag:354-413` — the Int corpus reused by probes 1, 2, 5.
- `docs/planning/v4-ci-overhaul-2026-05-30.md` — sibling architecture doc (§4 B1/B2/B3 diagnosis; SG-7 is orthogonal).
- `docs/planning/v4-ci-schema-worksheet-2026-05-30.md` — sibling §10.0-shape worksheet (Phase 1.5); §"Structural success criterion" cited above; §"Mechanical dispatch rule" inherited.
- `docs/planning/v4-correctness-ladder-2026-05-30.md` §10.0 — worksheet discipline source.
- `docs/planning/v4-leaf-model-verification-2026-05-30.md` §8 — fixture/grep-gate claim shape (probes 3, 4 follow-on).
- `src/v4/TASKS.md §T-22` — `T22-EVAL-CACHE-HASHES` feature anchor; close-status text references the 🟡 marker SG-7 dissolves.
- PR #3974 — interim CI relief; transport-wiring context that motivates SG-7's structural dissolution.
- PR #3938 §11.1 — SG-class single-authority approval discipline (manager-gated dispatch).
