# Design: T-30 hollow-alias structural gate (fact-density checker)

> **Status: DESIGN — map, not territory** (INVARIANTS.md "Map vs territory"). Nothing in this
> doc is landed *behavior* until the consumers in §8 execute green. This is the structural
> prerequisite for **T-4** (per-language primitive fact-bundles) and **G.1** (populating
> `PerLanguageFactBundleRegistry` in `v2.std.grounding`).
>
> Governing principles: **P1 Modeling Faithfulness** (hollow alias problem shape),
> **M1 Types are compositional facts** (`MODELING.md`), **Practice 8** structural tier
> (`docs/modeling-discipline.md` §8 — *Interim floor: the hollow-alias discriminator*).

## 1. Problem

A **hollow alias** is a type carrier that asserts an identity or shape while recording **zero
spec-read facts**. Two structural shapes cover the failure mode T-30 must block:

1. **Bare alias** — surface `type X = Y` lowers to a `TypeNode` whose connective is `Atom` or
   `Instantiation` with **no named edges** (only positional wiring, or no children at all).
2. **Positional-only `Conj`** — a record-shaped carrier whose edges are all `Positional`; it
   looks compositional but reads zero named facts.

These carriers are *structurally minimal*: they pass shape checkers (`well_formed`, arity
discipline) and survived review in the D2 era precisely because there was nothing there to be
wrong. The hollowness is invisible unless a reviewer asks "does the spec carry facts this
drops?" — which is why **convention** (Practice 8 interim three-prong review) is insufficient;
T-30 is the **structural** tier that makes hollow carriers **fail closed at compile time**.

**Downstream blast radius.** T-4 per-language primitive fact-bundles (Rust `i32`, C++ `int`, …)
require every external primitive to carry named spec facts. Without T-30, `type RustI32 = Int32`
lands in `extdeps/`, passes structural review, and blocks the G.0 registry from being the single
authority. T-30 is the hard gate that forces fact-bundle shape *before* G.1 population work
proceeds.

## 2. What already exists (M9 DFS — do not re-invent)

| Concept | Where it lives | State |
|---|---|---|
| `SourceSpecReadFact` classifier (`NamedFieldFacts` / `KernelAmbientAtom` / `NoFact` / `NotATypeCarrier`) | `src/v2/lens/fact_density.dag` | **landed shape**; gate fn authored |
| `carrier_spec_fact`, `named_fact_count`, `connective_spec_fact` | same | **landed shape** |
| `fact_density_hollow_alias_gate(node) -> Outcome<Witness<Node>>` | same | **landed shape**; rejects `NoFact` |
| `fact_density_lens` + `always_required_lenses()` | `src/v2/compiler/00_compile.dag` | **landed wiring** |
| Subtree enforcement (`fold_node` over inferred tree) | `run_required_lens_gates_on_subtree` in `00_compile.dag` | **landed wiring** |
| `HollowAliasGovernanceBar` (closed governance posture) | `src/v2/std/grounding.dag` G0.2 | **landed**; points at `v2.lens.fact_density` |
| Claim corpus (classify + enforce) | `src/v2/test/claim/lens_fact_density/*.dag` | **scaffold** — compile-only until T-22 execution |
| Rust bootstrap mirror (Practice 8 three-prong test IR) | `src/v3/compiler/src/v2_hollow_alias_gate.rs` | **P5(b) interim**; `dead_code` until `.dag` consumer |
| `fold_node` structural walk pattern | `src/v2/std/node.dag` (`well_formed`) | **landed** — subtree model to mirror |

**DFS conclusion:** no new ontology module is needed. T-30 is a **completion + execution**
problem on `v2.lens.fact_density`, not a greenfield design. The checker authority, classifier,
compile-lens adapter, and subtree driver already exist as `.dag` territory; what remains is
making that territory **execute** and dissolving the Rust mirror.

## 3. Structural hollow predicate (the generated checker spec)

T-30 implements **M1 fact-density only** — not the full Practice 8 convention-tier three-prong
AND (external-spec-primitive × bare-alias × no-coincidence-evidence). That semantic predicate
stays reviewer-hand for edge cases and is **out of scope** for the structural gate; coincidence
licensing for `std/` reuse is evidenced via G.0 registry rows and T-4 fact-bundles, not via a
parallel DECISIONS ledger.

### 3.1 Classifier: `carrier_spec_fact(carrier: Node) -> SourceSpecReadFact`

```text
match carrier.kind {
  ComputationNode → NotATypeCarrier          // pass-through; gate accepts
  TypeNode { connective: c } →
    if c is Atom && symbol_is_kernel_ambient(identity)
      → KernelAmbientAtom                  // exempt; gate accepts
    else
      → density_fact(named_fact_count(children))
        Zero   → NoFact                    // HOLLOW — gate rejects
        Succ+  → NamedFieldFacts { density }
}
```

`named_fact_count` counts edges whose label is `Named { name: _ }`; `Positional` edges contribute
zero. This single rule covers:

| Surface shape | Structural `Node` | `named_fact_count` | Verdict |
|---|---|---|---|
| `type X = Y` (alias) | `TypeNode { Instantiation }` + positional child | 0 | `NoFact` |
| bare atom type | `TypeNode { Atom { identity } }`, non-kernel | 0 | `NoFact` |
| positional-only record | `TypeNode { Conj }` + only `Positional` edges | 0 | `NoFact` |
| fact-bundle | `TypeNode { Conj }` + ≥1 `Named` edge | ≥1 | `NamedFieldFacts` |
| `Bool`, `String`, … (kernel) | `TypeNode { Atom }` + kernel symbol | 0 | `KernelAmbientAtom` |

**Explicit non-goals for v1:** rejecting a `Conj` whose named fields point at hollow *children*
while the parent itself has `NamedFieldFacts`. The parent carries facts (field names exist);
child hollowness is caught when the subtree walk visits the child. The nested-rejection claim
(`hollow_alias_nested_rejected.dag`) exercises this.

### 3.2 Gate: `fact_density_hollow_alias_gate(node) -> Outcome<Witness<Node>>`

```text
match carrier_spec_fact(node) {
  NoFact → outcome_rejected(Diagnostic { reason: fact_density_hollow_alias_locus, ... })
  _      → outcome_accepted(Holds { value: node })
}
```

Fail-closed on `NoFact` only. Diagnostic reason is a stable `Symbol` locus
(`^fact_density_hollow_alias_locus`) so claims can assert rejection without string matching.

### 3.3 Subtree closure (not inside the gate fn)

The gate is **local** (`Node -> Outcome`). **Global** "unconstructable" enforcement is the
compile adapter:

```text
validate_then_compile(...)
  → infer
  → run_required_lens_gates_on_subtree(inferred, always_required_lenses())
       // fold_node: run fact_density_lens on EVERY node in inferred tree
  → run_required_lens_gates(inferred, caller_supplied_lenses)
  → compile
```

`always_required_lenses()` prepends `fact_density_lens` even when the caller passes `lenses: []`
(`hollow_alias_vtc_empty_lenses_rejected.dag`). This is the mechanism that makes a hollow alias
nested under a named `Conj` child edge unconstructable — not merely the root.

**Design invariant:** do **not** fold subtree traversal into `fact_density_hollow_alias_gate`.
Subtree policy belongs to `v2.compiler.compile` (same separation as `well_formed` local check +
caller-driven full-tree walks). One gate fn, one fold site — cost-of-change = 1 when subtree
policy changes.

## 4. "Generated" checker — what that means here

In this codebase **generated** does **not** mean testgen emission of claims (T-22 owns claim
execution). It means:

1. **Authority in `.dag`** — the classifier and gate are substrate citizens in
   `v2.lens.fact_density`, not permanent hand-Rust.
2. **Lowered by bootstrap** — `lower_compile_module` prepends compiler/lens DAGs before the user
   module range (`strict_from`). Lens/compiler modules compile with bootstrap privileges (M1(2.8)
   opaque-body rejection applies only at `id >= strict_from`).
3. **Executed by a real consumer** — `apply_compile_lens` / `validate_then_compile` must call the
   **lowered** `fact_density_hollow_alias_gate`, not the Rust mirror.

The Rust file `v2_hollow_alias_gate.rs` is a **P5(b) interim mirror** with a *richer* test IR
(Practice 8 three-prong `HollowDeclarationSite`) than the structural `.dag` gate. It exists
because no production consumer yet executes the `.dag` gate body. **Dissolution trigger:** first
green execution of `fact_density_hollow_alias_gate` from lowered `.dag` in the compile-lens path;
then delete `v2_hollow_alias_gate.rs` and its SG-0 census row (−1
`EXPECTED_HAND_AUTHORED_NON_TEST` path in `sg0_census_test.rs` — the mirror is registered as
hand-authored **non-test** Rust, not `EXPECTED_HAND_AUTHORED_TEST`).

### 4.1 Why not `fold_node` inside the gate (yet)

`std/node.dag` notes `traverse_node` / Outcome-threading folds are **deferred until Ratified Q4**
(`std/diagnostic.dag`). The gate correctly uses a **local `match`** today; subtree closure is
already handled by `run_required_lens_gates_on_subtree`'s `fold_node` + `bind_outcome` chain in
`00_compile.dag`. When Q4 lands, optional refactor: a shared `traverse_outcome` helper — not a
blocker for T-30 closure.

## 5. Kernel-ambient exemption

Kernel-ambient atoms (`Bool`, `Char`, `String`, `Int`, `List`, `Map`, …) are legitimately
atomic — the spec carries no further facts to drop (`INVARIANTS.md` hollow-alias receipt;
`MODELING.md` M1).

**Current encoding:** `symbol_is_kernel_ambient(s: Symbol) -> Bool` closed disjunction over
`^fact_density_kernel_ambient_*` symbols in `fact_density.dag`. Phases A–D land with this
closed list as-is — no `target_model.dag` edit required.

**🟡 gated — dissolve-on-arrival (not Phases A–D):** `target_model.dag` also names kernel-ambient
carrier atoms (`symbol_kernel_type_atom`, `char_kernel_type_atom`, …) but that file is
translate's algebra home and is **mid-dissolution on #4699** (witty-raven). **HOLD:** do not
propose or land `target_model.dag` edits for T-30. After #4699 closes, unify the exempt symbol
set under a single `std/` authority (or derive exemption from kernel carrier nodes) so
`fact_density.dag` does not maintain a divergent ad hoc list long-term (P2 single-authority).
Until then, the two symbol namespaces may coexist; structural-tier enforcement is unaffected.

## 6. Boundary: structural vs convention tier

| Tier | Mechanism | Hollow criterion |
|---|---|---|
| **Structural (T-30)** | `fact_density_hollow_alias_gate` at compile | `carrier_spec_fact == NoFact` |
| **Convention (Practice 8 interim)** | PR review | bare-alias ∧ external-spec-primitive ∧ no-coincidence-evidence |
| **Governance (G0.2)** | `HollowAliasGovernanceBar` type | Declares posture; T-30 enforces |

T-30 deliberately does **not** encode "external spec primitive" or "coincidence evidence" —
those require extdeps context and G.0 registry traces. A bare `type InternalFoo = InternalBar`
between internal `std/` carriers is **not** `NoFact` if the carrier has named fields; if it is a
bare `Instantiation` with zero named facts, T-30 rejects it structurally even though Practice 8
convention might accept it (internal REUSE default). That is intentional tightening: M1 says
types decompose into facts; internal layers must still name facts, not bare-alias across
carriers.

## 7. Substrate-fact introduction (MODELING.md procedure)

- **Step 1 (DAG-ancestor):** ran. Classifier attaches to `v2.std.node` (`TypeNode`,
  `Connective`, `Edge`, `Named`/`Positional`) and `v2.std.nat` (`Nat` counting). No sibling
  module coined.
- **Step 2 (coproduct-vs-coordinate):** `SourceSpecReadFact` is a proper sum type
  (`NamedFieldFacts | KernelAmbientAtom | NoFact | NotATypeCarrier`) — alternatives, not flags.
- **Step 3 (consumer):** `CompileLens.gate` on `InferredTree.root` node via
  `fact_density_hollow_alias_compile_gate` adapter (projects `tree.root` — subtree handled
  separately as §3.3).

## 8. Consumers and falsification (E-10)

| Consumer | What it proves | Status |
|---|---|---|
| `lens_fact_density/hollow_alias_no_fact.dag` | bare Atom → `NoFact` | scaffold |
| `lens_fact_density/conj_positional_only_no_fact.dag` | positional Conj → `NoFact` | scaffold |
| `lens_fact_density/kernel_ambient_bool.dag` | kernel Atom → `KernelAmbientAtom` | scaffold |
| `lens_fact_density/fact_bundle_named.dag` | named Conj → `NamedFieldFacts` | scaffold |
| `lens_fact_density/hollow_alias_compile_lens_rejects.dag` | gate rejects hollow root | scaffold |
| `lens_fact_density/fact_bundle_compile_lens_passes.dag` | gate accepts fact-bundle root | scaffold |
| `lens_fact_density/hollow_alias_blocked_in_run_gates.dag` | `run_required_lens_gates` path | scaffold |
| `lens_fact_density/hollow_alias_blocked_via_always_required_lenses.dag` | `always_required_lenses` | scaffold |
| `lens_fact_density/hollow_alias_vtc_empty_lenses_rejected.dag` | caller `lenses: []` still blocked | scaffold |
| `lens_fact_density/hollow_alias_nested_rejected.dag` | subtree walk catches child hollow | scaffold |
| `manual/fact_density_anchor.dag` | v2-bootstrap compile anchor (3 carrier reads) | scaffold |
| **Target:** `validate_then_compile` executing lowered `.dag` gate | end-to-end enforce | **not yet** |
| **Dissolve:** `v2_hollow_alias_gate.rs` unit tests | absorbed by `.dag` claims above | interim |

**Done bar:** all `lens_fact_density` claims execute green via T-22 claim runner **and**
`fact_density_hollow_alias_gate` runs from lowered `.dag` in the compile lens path (not Rust
mirror). Typecheck + grep alone is not done (E-10).

### 8.1 Missing claim row (implementation should add)

| Claim | Why |
|---|---|
| `instantiation_bare_alias_no_fact.dag` | `type X = Y` lowers to `Instantiation`; implicit via zero named edges today but not explicitly claimed |
| `computation_subtree_passes.dag` | tree of `ComputationNode` only → compile succeeds (gate pass-through) |

## 9. Implementation phases (for the build slice — not this design PR)

### Phase A — Execute `.dag` gate (bootstrap bridge)

1. Ensure `v2.lens.fact_density` is in the compile bootstrap prepend set consumed by
   `lower_compile_module` (verify alongside `00_compile.dag` imports).
2. Wire lowered `fact_density_hollow_alias_gate` as the function pointer behind
   `fact_density_hollow_alias_compile_gate` (today: same-module `.dag` call; may need eval
   dispatch if not already linked).
3. Add integration test: `validate_then_compile` on hollow root → `Rejected` with
   `fact_density_hollow_alias_locus` **by execution**.

### Phase B — Claim execution (T-22 dependency)

4. Land T-22 generated claim execution for `lens_fact_density/*` corpus.
5. `fact_density_anchor.dag` compiles + reads execute in v2-bootstrap path.

### Phase C — Dissolution (P5(b) receipt)

6. Delete `src/v3/compiler/src/v2_hollow_alias_gate.rs`.
7. Remove `src/v3/compiler/src/v2_hollow_alias_gate.rs` from `EXPECTED_HAND_AUTHORED_NON_TEST`
   in `sg0_census_test.rs`; PR body Mechanism (b) −1 non-test path.

(Kernel-ambient symbol unification with `target_model.dag` is **out of Phases A–D** — see §5
dissolve-on-arrival mark; lands post-#4699, not in the build slice.)

### Phase D — Unblock T-4

9. G.1 RCA managers model per-language primitives as `Conj` fact-bundles with named edges.
10. `primitive_fact_bundle_for_subject` registry rows reference carriers that pass T-30 by
    construction.

## 10. Escalation triggers (STOP conditions)

- **Changing the hollow predicate** to include semantic prongs (external-spec, coincidence) inside
  `fact_density.dag` — belongs in G.0 / grounding spine, not T-30 structural tier. Escalate to
  grounding manager.
- **Touching load-bearing pipeline stages** (`emit`/`lower`/`infer`/`parse`) beyond the existing
  `always_required_lenses` / `run_required_lens_gates_on_subtree` wiring without an L2.5 model PR.
- **Editing `target_model.dag` for kernel-ambient unification** — HOLD until post-#4699 (§5);
  Phases A–D do not require it.
- **Adding a third outcome variant** to the gate — verdict stays `Outcome<Witness<Node>>`
  (Produced/Rejected per `std/diagnostic.dag`).

## 11. Architecture diagram

```text
  .dag model files                compile pipeline
  ─────────────────              ───────────────────────────────────

  TypeNode carriers              infer(source) → InferredTree
       │                                    │
       ▼                                    ▼
  carrier_spec_fact              run_required_lens_gates_on_subtree
  (classify local)                      │ fold_node over tree
       │                                ▼
       ▼                         per-node: fact_density_hollow_alias_gate
  NoFact / NamedFieldFacts /              │
  KernelAmbientAtom / NotATypeCarrier     ├─ NoFact → Rejected (STOP)
       │                                  └─ else  → Accepted
       ▼                                    │
  fact_density_hollow_alias_gate            ▼
  (Node → Outcome)               compile_inferred (only if gates pass)
```

---

**Hand-off:** This design ratifies the existing `v2.lens.fact_density` + `00_compile.dag` wiring
as the T-30 authority. The build slice owns Phase A–C execution and T-22 claim green; T-4 waits
on Phase D after Phase A is executed-green.
