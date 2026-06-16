# Coproduct-arm reflection — design + ban-lift proposal

> **Status: design (Phase 1 of the R-reflect lane). GATED.**
> This document is a proposal. No implementation, primitive, or consumer
> migration lands until it has (a) substrate **DESIGN-SIGN** (snappy-crab-849)
> and (b) the operator's **BAN-LIFT RULING** (routed via zesty-swift-79). It
> exists to be reviewed, not to authorize work.
>
> **Anchor.** This is the *Track 2 — substrate-derivation* spec for the lens
> family the dissolve-on-arrival marks cite as `docs/design-dissolution-lens.md
> L1.1` ("coproduct reflection / variant-enumeration substrate"). That internal
> doc was removed in the public-visibility flip (#4192); this public design doc
> is its surviving, scoped successor. Phase 3 re-anchors the marks' citations
> here.
>
> **Cross-refs (live):** INVARIANTS.md P1/P2/P3/P4/P5 and §"Reflection evidence
> is not structural proof"; MODELING.md M8/M9/M11/M12; THESIS §"Self-inspection",
> §"Structural decompression"; src/v3/SELF_HOSTING.md §2.3 (language-agnostic
> core); CODING.md (Rust style for any compiler support).

---

## 0. One-paragraph summary

A **coproduct-arm reflection** primitive enumerates a closed sum type's variant
arms and projects per-arm keys/labels, so downstream models **derive** variant
lists, arm-discriminants, and item rosters **from the type** instead of
hand-mirroring them. Today ~15 `gunbc#4759`-bound mirror marks (plus ~12
broader-family arm-tables) hand-maintain second copies of facts the declaration
already states. Reflection makes those mirrors *unrepresentable* (you cannot
drift from yourself) for structurally-determined facts, and converts the
genuinely per-arm-semantic consumers into **fail-closed total maps keyed by the
reflected arm set**. The reflection *mechanism itself* is held honest by one
tree-wide **by-execution conformance gate** (reflection output == an
independently-derived arm set), which replaces N hand-mirrors with a single,
broader, executed check. The ban is lifted **only** for this mechanism, under
those guarantees.

---

## 1. The problem — hand-rostered / mirror marks

A coproduct is declared once:

```dag
type Connective = Atom { identity: Symbol } | Conj | Disj | Arrow | Cardinality | Instantiation   // std/node.dag
```

…and then its arm set is **re-stated by hand** in many places. Grounded census
(verify line numbers at HEAD; from `main`):

**Core falsifier — 15 `gunbc#4759`-bound mirror mark-lines across 6 files:**

| File | Mark(s) | What it mirrors |
|---|---|---|
| `src/v2/std/node.dag` | :46 | `ConnectiveCoproductVariant` enum + `connective_coproduct_variant_keys()` + `behavior_coproduct_variants()` — hand-lists of `Connective`/`Behavior` arms |
| `src/v2/std/verification.dag` | :93 | `impossible_bug_class_coproduct_variants()` — hand-list of `ImpossibleBugClass` arms |
| ″ | :134 | `impossible_bug_class_from_diagnostic_reason()` — reason→class bridge |
| ″ | :195/:197 | `TestClaimCoproductVariant` — key-enum mirror of `TestClaim` arms |
| ″ | :210/:212 | `ClaimAnchorKey` manual/generated union (anchor split) |
| ″ | :275 | `test_claim_label()` — hand-enumerated `TestClaim`→label projection |
| ″ | :287 | `test_claim_coproduct_variant()` — hand-enumerated `TestClaim`→key projection |
| `src/v2/test/claim/generated/coproduct_exhaustiveness.dag` | :53/:63/:92 | hand-rolled arm-discriminant checks (DiagnosticClaim / GeneratedClaimAnchor / 4-arm `TestClaimCoproductVariant` equality) |
| `src/v2/test/claim/generated/algebra_law_conformance.dag` | :32 | local nat-expression node shape-tags |
| `src/v2/lens/coverage.dag` | :947 | impossible-bug reason membership (consumes verification's projection) |
| `src/v2/test/claim/manual/manual_corpus_roster.dag` | :4 | "item-registry reflection replaces explicit import roster" |

**Broader family (~12, same shape, not #4759-bound):** arm-tables /
variant-discriminant marks in `lens/coverage`, `lens_cost/*`, `lens_complexity/*`
(per-arm cost & complexity tables). Surfaced as a Phase-3 consumer cluster;
lower priority than the core.

**The ban line itself:** `src/v2/test/claim/extdeps/coordination_claims.dag:18`
— "reflection banned; decl-shape mirrors are 2FA-for-code."

**Why this is debt (live invariants).** Each mirror is a *second authority* for
a fact the declaration already owns — INVARIANTS P2 "Parallel authority" and
MODELING M2/M7. Cost-of-change is > 1: adding an arm to `Connective` forces edits
in `node.dag`, `coverage.dag`, and any roster keyed on it. The marks are valid
🟡 only because they bind a named dissolution trigger — *this* primitive
(P5 "Scaffold without dissolution trigger"). `gunbc#4759` is a **closed
label-hygiene re-anchor, not prior art** — the capability is greenfield.

---

## 2. The ban — read carefully, then steelmanned

`coordination_claims.dag:18` bans reflection because **decl-shape mirrors are
"2FA-for-code"**: a second, *independently authored* statement of the arm set
that catches drift. Naively deleting the mirrors and deriving from the
declaration removes that second factor. The design must not hand-wave this.

The 2FA protects against **two distinct drift classes**, and they have
*different* answers:

- **(ii) declaration ↔ consumer drift.** A consumer's variant handling silently
  goes stale when an arm is added (the classic non-exhaustive-handling bug). The
  mirror catches it because a hand-written exhaustive `match` over the coproduct
  *fails to compile* until updated — that compile error is the real "second
  factor."
- **(i) declaration ↔ mechanism drift (self-hosting).** The deeper concern: the
  compiler is partly self-hosted (`.dag` → emitted stage0 Rust). If the
  reflection *mechanism* has a bug — drops an arm, miscounts under a generic
  instantiation, the lowering is wrong — a consumer relying solely on reflection
  silently inherits the wrong arm set, and nothing catches it, because the only
  authority is the broken one. The hand-mirror is an **independent witness** of
  the same fact, authored by a human reading the declaration; comparing the two
  catches mechanism bugs.

Both are real. The design answers each — and the answers are *stronger* than the
mirrors, not weaker.

---

## 3. Where reflection attaches (M9 — DFS the concept DAG)

**No new top-level concept. No substrate extension.** A coproduct is *already*
a substrate citizen: in `std/node.dag` a sum type is a node with
`kind: TypeNode { connective: Disj }`, and `connective_edge_discipline(Disj) =
LabeledEdges` — so **its arms are its named-edge children** (`Edge { label:
Named { name: armLabel }, target: armPayload }`). Enumerating arms is therefore
a *structural query over a `Disj` node's edges* — the same shape as the existing
`named_edge_target_lookup`, `node_subtree_nodes`, `all_edges_named` in
`std/node.dag`. The v3 reflection already has `disj_variant_ty` ("look up a
variant type in a Disj coproduct by label"), confirming arms are label→payload
edges.

So the primitive **attaches to `v2.std.node`** as a derived operation over the
existing `Disj` connective. INVARIANTS P2 prescribes extending substrate
reflection *before* migrating consumers; here that means adding the capability to
**v2 `.dag` substrate** (`std/node.dag`), not extending the v3-Rust
`substrate_reflection` seed (`lens_declaration_apply.rs` — see §4.3). INVARIANTS
§"Reflection evidence" (:385) "extend the `substrate_reflection` submodule"
refers to that v3 seed submodule; this lane's deliverable is the v2 analogue. It
adds **no 7th connective and no 6th behavior**, so it does not trip the THESIS
substrate-extension stop-signal.

This is the **Track 2 cure** named in the L1.1 anchor: "the substrate lets you
*declare* an algebraic type but does not *derive its canonical operations*, so
workers hand-roll discriminants and catamorphisms." Coproduct-arm reflection is
the missing derived operation.

---

## 4. The sanctioned mechanism

### 4.1 Reflected fact carrier (std/node.dag)

```dag
// A single reflected arm of a closed coproduct (Disj node).
type CoproductArm {
  label: Symbol        // the arm's name (the Named edge label)
  payload: NodeShape   // the arm's payload type; NoEdges-shaped for nullary arms
}
```

`CoproductArm` is a fact-bundle (named edges, M1) — it carries the two facts the
declaration states about an arm and nothing more.

### 4.2 Core projections (compile-time, deterministic, closed-only)

- **`coproduct_arms(T) -> List<CoproductArm>`** — the full reflected arm list of
  a *closed, declared* coproduct type `T`, in declaration order.
- **`coproduct_arm_keys(T) -> List<Symbol>`** — `map(coproduct_arms(T), .label)`.
  The key projection that replaces every hand-written key-enum + key-list mirror
  (`ConnectiveCoproductVariant`, `TestClaimCoproductVariant`, …).
- **`coproduct_nullary_inhabitants(T) -> List<T>`** — for coproducts whose arms
  are **all nullary** (no payload), the value-level inhabitant list. This is the
  total replacement for `behavior_coproduct_variants() -> List<Behavior>` and
  `impossible_bug_class_coproduct_variants()`. For a coproduct with **any
  payload-carrying arm**, this **fails closed** with a typed diagnostic — the
  primitive never fabricates an inhabitant for a payload arm (C-1/C-9; a default
  payload would be a fabrication). Payload-arm coproducts get keys, not values.
- **`common_field_projection(T, field) -> ...`** *(Phase-2b — split out per
  design-sign Q2; lands after C1 + the corrected gate are green on
  Connective/Behavior).* When *every* arm of `T` carries a field of the same name
  and type (e.g. `TestClaim.label`, `TestClaim.anchor`,
  `TestClaim.classification`), the field is projectable generically. This
  replaces `test_claim_label()` and the `test_claim_*`-family common-field
  hand-projections. If an arm lacks the field, it fails closed (the field is not
  common).

**Discipline of the primitive** (all enforced, not aspirational):

1. **Compile-time only.** `T` is a statically-known, closed type; the arm set is
   resolved from its declaration at compile time. No runtime reflection. No
   open-set reflection (M4 — open sets stay diagnostics, not enumerations).
2. **No execution semantics, no narrowing.** Same posture as the blessed
   `reflect_program_dag_nodes_in_file`: a complete, deterministic structural
   projection, no per-consumer narrowing (INVARIANTS §"Reflection evidence").
3. **Fail-closed everywhere.** Non-coproduct `T`, open `T`, missing common
   field, or value-inhabitant request on a payload arm → typed diagnostic via
   `Outcome`, never a fabricated default (P3).

### 4.3 Compiler support (Phase-2 scope; named here, not built)

The primitive requires the compiler to resolve a type reference `T` at its call
site to the declaration's `Disj` node and project its arm edges. This
resolution-and-projection is the **primary Phase-2 build risk** and is exactly
what the §5 conformance gate protects.

**Layer-boundary precision (design-sign condition 2).** This is **new v2
substrate**, *analogous to* — not an extension of — the existing v3-Rust
`substrate_reflection` submodule (`lens_declaration_apply.rs`). That v3 surface
is **bootstrap seed**: per src/v3/SELF_HOSTING.md the seed *shrinks* while v2
substrate *grows*. The v2 primitive is authored as v2 `.dag` substrate + v2
compiler support; it does not inherit from or call the v3 seed. The v3 reflection
is relevant only as (a) a shape precedent and (b) a *cross-generation* check
factor in the gate (§5.2), never as the home of the new capability.

---

## 5. The 2FA crux — preserved, and strengthened

### 5.1 Class (ii) drift is *eliminated*, not caught

For structurally-determined facts (arm lists, keys, common-field projection,
structural equality / discriminant), reflection removes the *second list
entirely*. There is nothing to drift from — the consumer derives from the
declaration. INVARIANTS P2: "If the canonical answer doesn't exist yet, extend
the substrate reflection first, then migrate consumers." Making drift
*unrepresentable* strictly dominates *catching* it. **No 2FA replacement is
needed for class (ii); the drift class is dissolved.**

### 5.2 Class (i) drift — the by-execution conformance gate (corrected per design-sign Q1)

The mechanism-drift concern is answered by a **by-execution conformance gate**:
for every gated coproduct, the reflection output must equal an
**independently-derived** arm set, where "independent" means *derived through a
distinct lowering path* — not merely "a different function." A bug that corrupts
the reflection builtin must NOT also corrupt the witness, or the gate is
**circular** and passes falsely (the self-hosting class-(i) drift the ban
guards). The factors below are graded by that independence test.

**Why the obvious factor fails (the load-bearing correction).** The natural
candidate — v2 type-checker match-exhaustiveness — is **NOT independent on its
own.** v2 inference and the v2 reflection builtin **both resolve `T` to the same
parsed `Disj` node**; a common-mode resolver/parse arm-drop corrupts *both*
sides, and the gate passes falsely. So exhaustiveness-count is **common-mode with
the thing it is checking** and cannot be the gate's independent factor.
Similarly, the v3 `reflect_program_dag_nodes_in_file` posture is **not a
pre-existing factor**: it is v3-Rust *seed*, partial, and INVARIANTS:381-383
explicitly tags it "not yet a closed mechanical theorem … not a substitute for a
future full conformance gate." It is the gate we are **building**, not one we can
lean on.

**The gate (REQUIRED).** For every gated coproduct, at least one of the
following *path-distinct* witnesses must hold by execution:

- **Path 3 — syntactic arm key-set (primary, required for non-roster
  coproducts).** A parser/syntactic derivation of the arm **labels** — reading the
  `|`-separated arm names at the grammar level, *before* the shared resolver the
  reflection builtin uses — and **set-comparing** them to `coproduct_arm_keys(T)`.
  Because it is computed on a distinct (syntactic) path, a resolver/lowering bug
  in the reflection builtin does not corrupt it. A cardinality count alone would
  **not** suffice: `|children| == |keys|` passes falsely on arity-preserving
  relabel (wrong key, right count) — squarely inside class-(i) mechanism drift.
  Key-set equality catches drop, add, and relabel. This is the genuine second
  factor.
- **Cross-generation check (where v3 covers the carrier).** For carriers the v3
  seed reflects, `v2 coproduct_arm_keys(Behavior) == v3 reflect_behavior_list`
  arm set. Two generations, two independent code paths — a strong factor for the
  Behavior/Connective carriers v3 already covers, usable in addition to Path 3.
- **Frontier-discovery equality (roster clusters only — the D1c seam).** The CI
  roster cluster has `affected_set_ci_runner.dag:287`
  `ci_runner_selected_matches_frontier_discovered`: "discovered == roster:
  selection output must equal explicit frontier discovery, not hand-roster
  laundering." For those clusters the witness is **reflection-generated roster ==
  independently frontier-discovered set** — naturally path-independent
  (consumer-side derivation) and the strongest factor *there*, but roster-scoped,
  not a general gate.

**Explicitly rejected as the gate's independent factor:** v2 exhaustiveness-count
alone (common-mode), and the v3 reflection posture alone (the gate being built).
The exhaustiveness check may still run as a *consistency* signal, but it does not
discharge the independence obligation.

This corrected gate is **strictly stronger** than the hand-mirrors: it is checked
**by execution**, it covers **every** gated coproduct via a genuinely
path-distinct witness, and it is **one** gate to maintain instead of N driftable
lists. The mirrors were a weaker, partial, by-inspection form of it.

### 5.3 Per-arm semantics — fail-closed total maps keyed by the reflected set

Some consumers are **not** structurally determined: they assign a value *per arm
that is not recoverable from the arm's shape* (e.g.
`impossible_bug_class_from_diagnostic_reason`, per-arm cost/complexity tables).
Reflection cannot derive these, and silently defaulting them would be a C-8/M5
fabrication. These do **not** dissolve into pure reflection. Instead they
**reuse the existing `TotalMap<K,V>`** (`std/collection.dag:165`, `lookup:
fn(K) -> V`) — *no* new carrier (declaring a `TotalArmMap<T,V>` parallel to
`TotalMap` would be the very parallel-authority anti-pattern this lane kills, per
design-sign Q3 / M9). "Keyed by the reflected set" is **not a type** — it is a
**conformance witness**: `domain(map) == coproduct_arm_keys(T)`. A
smart-constructor that builds a `TotalMap` and emits that domain-equality witness
is the sanctioned shape. The consumer supplies the codomain; a missing key is a
**typed diagnostic at build time**, not a silent fall-through (P3 "Case
enumeration for open sets" → typed table; M7 data-table single-authority).

This **preserves and strengthens** the class-(ii) exhaustiveness 2FA: "did you
handle the new arm?" moves from "a hand-written `match` fails to compile" to
"the substrate proves the table is total against the canonical arm set." Adding
an arm grows the reflected domain → the table is now missing a key → fail-closed.
The reviewer still consciously supplies the new arm's semantics — but there is no
longer a *second arm list* to forget.

---

## 6. Consumer taxonomy (drives Phase 3)

Every census entry sorts into exactly one bucket:

| Bucket | Mechanism | Census members |
|---|---|---|
| **C1 — pure enumeration** (arm list / keys / nullary inhabitants) | `coproduct_arm_keys` / `coproduct_nullary_inhabitants` + conformance gate | `connective_coproduct_variant_keys`, `behavior_coproduct_variants`, `impossible_bug_class_coproduct_variants`, `TestClaimCoproductVariant` key-enum + `test_claim_coproduct_variant`, `coproduct_exhaustiveness.dag` arm checks |
| **C2 — common-field projection** (field present on every arm) | `common_field_projection` *(Phase-2b)* | `test_claim_label` and the `test_claim_*` common-field family |
| **C3 — structural equality / discriminant** | derived discriminant over arms (L1.1 lens target) | `connective_eq`/`behavior_eq`/`test_claim_coproduct_variant_eq` (broader family) |
| **C4 — per-arm semantics** (value not in the shape) | fail-closed **`TotalMap<K,V>`** (std/collection.dag:165) + `domain == coproduct_arm_keys(T)` witness | `impossible_bug_class_from_diagnostic_reason`, `coverage.dag:947` reason membership, `lens_cost/*` & `lens_complexity/*` arm tables |
| **C5 — item rosters** (lists kept in sync with a registry/frontier) | reflection-generated roster + **D1c :287 equality witness** | `manual_corpus_roster`, `affected_set_ci_runner` / `affected_testgen_ci_runner` / `lens_ownership/subject_roster` (swift-stag-552 D1c) |

C4 honestly does **not** vanish into pure reflection — it dissolves the *arm
list* duplication (the total-map's domain is reflected) while keeping the
genuine per-arm decision as single-authority data. Naming this honestly is part
of the design: the falsifier "marks → 0" is met because each mark's *mirror*
dissolves, even where a (now fail-closed, single-authority) semantic table
remains.

---

## 7. The ban-lift (the exact ask)

**Lift the `coordination_claims.dag:18` reflection ban for, and only for, the
following mechanism:**

> Deterministic, **compile-time** reflection over a **closed, declared
> coproduct** type that projects its **arm keys, common fields, and (for
> all-nullary coproducts) value inhabitants**, where:
> 1. every consumer migration deletes its own dissolve-on-arrival mirror in the
>    **same PR** (per each mark's forbidden-clause);
> 2. the mechanism is covered by a **tree-wide by-execution conformance gate**
>    (§5.2) that proves reflection output equals an independently-derived arm
>    set; and
> 3. **per-arm-semantic** consumers (C4) become **fail-closed total maps keyed
>    by the reflected arm set**, never silent defaults.

**Stays banned** (explicitly out of scope of this lift):
- runtime / dynamic reflection;
- reflection over **open** sets (M4 — open sets are diagnostics, not enumerable);
- fabricating inhabitants for **payload-carrying** arms (C-1/C-9);
- using reflection to **silently default** per-arm semantics (C-8/M5);
- any reflected projection **not** covered by the §5.2 conformance gate (an
  ungated reflection consumer is exactly the single-broken-authority hazard the
  ban was protecting against).

Rationale for a *scoped* lift rather than a blanket repeal: the ban's protective
intent (2FA against mechanism drift) is **satisfied by construction** within
this scope (§5) and **violated** outside it. The lift encodes the ban's own
anticipated "proper mechanism," not its repeal.

---

## 8. Invariant alignment

- **P1 Faithfulness.** Arms are read from the declaration; no heuristic, no
  ungrounded fact. The primitive is the missing derived operation, not a second
  authority.
- **P2 Boundary.** Single authority (the declaration); parallel mirrors deleted
  in the same change; reflection exposed through the declared `std/node` surface.
- **P3 Fail-closed.** Non-coproduct / open / payload-inhabitant / missing-common-
  field / missing-total-map-key all produce typed diagnostics; no fabrication.
- **P4 Decidability.** Compile-time, bounded fold over a finite Disj node's
  edges; terminates by construction. No new connective/behavior; no runtime
  capability.
- **P5 Dissolution.** Net mirror count strictly decreases; every migration PR is
  debt-negative; the falsifier (15 core + ban line + ~12 family marks) → 0.

---

## 9. Phasing

- **Phase 1 (this doc).** Design + ban-lift proposal. Gate: snappy-crab-849
  DESIGN-SIGN + operator BAN-LIFT ruling.
- **Phase 2a (gated).** Build `CoproductArm` + `coproduct_arms` /
  `coproduct_arm_keys` / `coproduct_nullary_inhabitants` in `std/node.dag` (new
  v2 substrate) + the compiler resolution support, and the **corrected §5.2
  conformance gate** (Path 3 syntactic arm key-set + cross-generation where v3 covers).
  Inhabit on real coproducts first (`Connective`/`Behavior`) with the gate green
  by execution before any consumer migrates (facts-before-abstraction).
- **Phase 2b (gated, after 2a green).** Add `common_field_projection` (the
  `test_claim_*` common-field family's path).
- **Phase 3 (gated, may spawn lean per-cluster workers).** Migrate consumers
  cluster-by-cluster in taxonomy order: C1 std mirrors → C1/C3 generated corpus
  exhaustiveness/algebra → C5 manual corpus roster → C5 CI rosters (swift-stag-552
  D1c, with the :287 equality witness). Each PR deletes its own
  dissolve-on-arrival marks. Re-anchor the marks' `design-dissolution-lens.md
  L1.1` citation to this doc.

---

## 10. Design-sign resolutions (snappy-crab-849, CONDITIONAL sign — issuecomment-4700373336)

1. **Conformance-gate independence — RESOLVED (the load-bearing condition).**
   v2 exhaustiveness-count alone is **NOT** an independent factor — it is
   common-mode with the reflection builtin (both resolve `T` to the same parsed
   `Disj` node), and the v3 reflection posture is the gate being *built*, not a
   pre-existing factor. **REQUIRED:** the gate's independent factor is the
   **syntactic arm key-set (Path 3)** — `|`-separated arm labels from the parse
   tree set-compared to `coproduct_arm_keys(T)`, not a cardinality count — for
   every gated coproduct, and/or the **cross-generation check** (`v2
   coproduct_arm_keys == v3 reflect_behavior_list`) where v3 covers the carrier;
   rosters additionally use the frontier-discovery equality. Rewritten in §5.2.
   Exhaustiveness may run as a consistency signal but does not discharge
   independence.
2. **Common-field projection — RESOLVED: split to Phase-2b.** Land C1 + the
   corrected gate on `Connective`/`Behavior` green-by-execution first; add
   `common_field_projection` after. Reflected in §4.2 / §6 / §9.
3. **C4 carrier — RESOLVED: reuse `TotalMap<K,V>`** (`std/collection.dag:165`).
   No `TotalArmMap`. "Keyed by reflected set" is a conformance witness
   (`domain == coproduct_arm_keys(T)`), not a new type; smart-constructor emits
   the witness. Reflected in §5.3 / §6.
4. **Value-inhabitant boundary — CONFIRMED.** All-nullary only;
   payload arms → fail closed; no inhabit-with-holes.

**Doc-precision condition (condition 2) — APPLIED:** the primitive is **new v2
substrate**, analogous to (not an extension of) the v3-Rust `substrate_reflection`
seed, which shrinks while v2 grows (§4.3).

**§5.2 re-read — CONFIRMED (snappy-crab-849, post-0071c645):** design-sign
carries on the corrected gate. Binding sharpening applied: Path 3 is a **syntactic
arm key-set** (label set-compared to `coproduct_arm_keys(T)`), not a cardinality
count — count alone passes falsely on arity-preserving relabel. With Path 3 as
key-set, **no third path is required** for non-roster/non-v3 coproducts; it
discharges the independence obligation alone. Cross-generation is additive where
v3 covers the carrier; frontier-discovery is roster-scoped only. The 2FA guarantee
proves in Phase-2 when Path 3 (key-set) lands GREEN before any consumer migrates.

**Operator gate (still open):** BAN-LIFT ruling (routed via zesty-swift-79),
predicated on this corrected 3-factor gate.
