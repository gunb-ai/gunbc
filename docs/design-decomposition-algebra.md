# Decomposition Algebra — small modeling project

**Status**: DRAFT (2026-05-12). Authored to formalize the project-side replanning protocol the operator described. Iteration expected.

**Premise (operator-supplied)**: when you decompose a goal — `make a C compiler` = `½·frontend + ½·backend` — you are making an *algebraic claim* that the children jointly preserve the parent's meaning. When integration fails ("this isn't a C compiler"), you are in *contradiction*: by construction-of-parts you have it; by failing integration you don't. Resolution requires walking back the decomposition DAG to find where meaning was dropped.

This doc is the small modeling project: a procedure for the walk-back, a type sketch for the algebra (gunbc-lens-style), and a sketch of how dashboard comms (PM↔Mgr↔Worker) map onto the structure.

---

## §1. The contradiction-trigger

At any node `N` in a decomposition DAG, two algebraic claims hold simultaneously:

- **A**: `N = Σ c_i · child_i(N)` — N's meaning is the sum of its children's meanings, with intersubjective shares `c_i` summing to 1.
- **B**: `meaning(N)` is whatever the parties have agreed it is — the *external* claim, often verified by witnesses (integration tests, prose attestations, structural lenses).

Walk-back fires when **A and B disagree at some node**:

- **(a) Algebraic imbalance** — children don't cover the parent's meaning (`Σ child_meanings ⊊ parent_meaning`). Some part of `N` is unaccounted for.
- **(b) Claim contradiction** — `N` is asserted to NOT hold its meaning, even though A says it should (the integration test fails; the reviewer says "this isn't X").

Both surface the same root cause: the decomposition's algebra is broken at or above the trigger point.

---

## §2. Types (gunbc-lens sketch)

The algebra in gunbc-style types. These would eventually live in `dsl/std/` as a `MeaningDecomposition` lens analogous to the Cost lens (per `feedback_lenses_not_passes`: derivable from physics, no heuristics).

### 2.1 `Node`

```
type Node {
  claim: Claim
  decomposition: Decomposition?    // None = leaf
  witnesses: List<Witness>
}
```

A node is the atomic unit of the DAG. Carries its agreed claim, optional decomposition into children, and witnesses supporting the claim.

### 2.2 `Claim` — intersubjective meaning

```
type Claim {
  text: String                     // prose statement: "a C compiler"
  parties: Set<Party>              // who has attested this claim's meaning
  evidence_for_match: List<Evidence>  // why the parties believe the claim holds
}
```

A claim is meaningful only relative to the parties that have attested to its meaning. The rock-vs-compiler case is a meta-failure: two parties hold different `meaning(Claim.text)`, so the claim's identity is itself contested. Decomposition can only proceed when the **root-asker** accepts the claim; other parties may trigger walk-back if their meaning of the claim diverges from the root-asker's.

### 2.3 `Decomposition` — the algebraic statement

```
type Decomposition {
  parent: Node
  children: List<Node>
  shares: List<Share>              // c_i for each child; intersubjective; sum = 1
  receipt: BalanceReceipt          // audit trail of "why we believe coverage"
}

type Share {
  child_index: Int
  coefficient: Rational            // c_i ∈ (0, 1]
}

type BalanceReceipt {
  attesting_parties: Set<Party>
  attestation_text: String         // "these N children cover all of P, per X/Y/Z"
  witnesses: List<Witness>         // optional structural witnesses (integration tests, etc.)
}
```

The `shares` are NOT effort-weights or priority-weights. They are the parties' agreed structural shares: "this much of P's meaning is held by this child." Coefficients are intersubjective declarations, not measurements. They sum to 1 by construction.

### 2.4 `Witness` — evidence that a claim holds

```
type Witness =
    IntegrationTest { name: String, passes: Bool }
  | ProseAttestation { party: Party, text: String, timestamp: Timestamp }
  | StructuralLens { lens_name: String, computation: LensComputation }
```

Three kinds, ordered by structural strength (`StructuralLens` strongest, `ProseAttestation` weakest). Gunbc-discipline preference: `StructuralLens > IntegrationTest > ProseAttestation`. Prose alone is the weakest receipt and most failure-prone.

### 2.5 `WalkBackEvent` — the procedure encoded

```
type WalkBackEvent {
  trigger_node: Node
  trigger_kind: AlgebraicImbalance | ClaimContradiction
  triggering_party: Party
  visited_ancestors: List<Node>
  resolution: WalkBackResolution
}

type WalkBackResolution =
    StableAncestor(Node)             // re-decompose subtree below this node
  | RootContested(Node, Party)       // root claim itself disputed; escalate to root-asker
```

---

## §3. Procedure: walk-back algorithm

```
function walk_back(trigger: Node, party: Party, kind: TriggerKind) -> WalkBackResolution:
  current = trigger
  visited = []

  while not is_root(current):
    parent = parent_of(current)
    visited.append(parent)

    # Algebraic check: do parent's children still sum to parent's claim?
    if not children_balance(parent):
      current = parent
      continue

    # Intersubjective check: does the triggering party still agree with parent's claim?
    if not parties_agree(parent.claim, party):
      current = parent
      continue

    # Both checks pass at parent — found the stable ancestor.
    return StableAncestor(parent)

  # Walked to root without finding stability.
  return RootContested(current, party)
```

**Resolution (post-walk-back)**:

- **`StableAncestor(A)`**: A is the rebalance point. Re-decompose the subtree rooted at A with the new understanding. All affected parties must re-attest the new `Decomposition`. The children below A are invalidated until the new decomposition is in place.
- **`RootContested(root, party)`**: the root claim's meaning is itself disputed. Surface to the root-asker (operator). Either (a) party adjusts their meaning to match the root-asker's, or (b) root-asker re-states the root claim. No work below the root can proceed until resolution.

---

## §4. Worked example — PR #2745 misread (2026-05-12)

Live case study. The walk-back protocol applied retroactively to a real recent contradiction.

### 4.1 Decomposition DAG (implicit, as it stood)

```
Root: operator intent "T-WAD FULL R3-close" (per operator directive 2026-05-12)
├── PM scope doc (PR #2744 §0 criteria 1-5)
│   ├── Criterion 3: "WorkflowRuntime toggle — WorkflowRuntime + project_github_actions in gunbc-substrate"
│   │   ├── §1.8 gate 99: workflow_runtime_open_enum_landed
│   │   ├── §1.8 gate 100: project_github_actions_landed
│   │   └── WI-2 brief: NEW file dsl/gunbc/ci_emission.dag declaring enum + signature + Practice 4 receipt
│   │       └── cool-carp-720 worker → PR #2745
│   └── ... (other criteria, omitted)
└── ... (other Mgr lanes, omitted)
```

Shares (operator-attested at root): `T-WAD = ⅖·Criterion1 + ⅕·Criterion2 + ⅕·Criterion3 + ⅒·Criterion4 + ⅒·Criterion5` (illustrative; actual shares are intersubjective).

### 4.2 The contradiction

At ~15:18Z, PM (deep-wolf-155) audited substrate state post-PR-#2745-merge and found:
- WI-2 brief claim: PR #2745 will declare `WorkflowRuntime` enum + `project_github_actions` signature in `dsl/gunbc/ci_emission.dag`
- Actual delivery: PR #2745 created `dsl/extdeps/github/ci.dag` (platform substrate for workflow file location, with Practice 4 receipts) + modifications to `actions.dag` + comment additions to `dsl/gunbc/ci.dag`. **`dsl/gunbc/ci_emission.dag` was never authored.**

**Trigger**: Claim contradiction at the cool-carp-720 PR #2745 node. Brief claim said deliverable X; reviewer-audit said delivery was Y; X ≠ Y.

### 4.3 Walk-back trace

```
Start: PR #2745 node (claim: "delivers WI-2 substrate")
  → Check parent (WI-2 brief node):
       claim: "NEW file dsl/gunbc/ci_emission.dag with WorkflowRuntime + project_github_actions"
       Does this claim still hold? YES (brief is unchanged; meaning intact)
       Does PR #2745 (child) balance to this claim? NO (delivery ≠ brief)
       → STABLE ANCESTOR found: WI-2 brief node.

Resolution: re-decompose subtree below WI-2 brief. cool-carp-720 archived; original WI-2 scope is now first-in-queue for warm-wolf-698's lane (substrate reattempt).
```

### 4.4 What the procedure caught vs. what prose-driven workflow caught

- Prose-driven (actual): PM made a *wrong claim* to Director ("gates 99 + 100 may close via PR #2745"); audited only after Director questioned; grep-discovered the gap; surfaced as PM execution error.
- Procedure-driven (hypothetical): when PR #2745 closed, the algebra check at WI-2-brief-node would have flagged "children's deliverables (PR #2745's files) do not include the claimed `dsl/gunbc/ci_emission.dag`" — gap visible structurally without PM audit.

The procedure surfaces the gap **at PR-close time**, not at audit-time. Saves the wrong-claim cycle.

### 4.5 Cost of the procedure

For this case, the algebra requires:
- WI-2 brief node explicitly lists `dsl/gunbc/ci_emission.dag` as a delivery item
- PR #2745 close-event triggers an algebra check: do delivered files cover the listed deliverables?
- Mismatch raises a `WalkBackEvent` automatically

This is structurally enforceable — file-list intersection is a `StructuralLens`. Strongest witness class.

---

## §5. Application sketch — dashboard comms as decomposition DAG

(High-level; iteration with operator expected.)

The PM↔Mgr↔Worker dashboard messages we already exchange are *implicit walk-back signals*. Each is a triggering event for some `WalkBackEvent`. Examples:

| Today (prose) | Decomposition-algebra mapping |
|---|---|
| Reviewer BLOCKING on PR | Trigger at PR node; claim contradiction or algebraic imbalance |
| Worker STOP/PING to Mgr | Trigger at worker brief node; "I can't satisfy this claim" |
| Mgr "scope mismatch" to PM | Trigger at scope doc node; "brief children don't sum to scope" |
| PM correction to Director | Trigger at scope-doc-claim node, walks back to operator-intent root |
| Operator BLOCKING on PR thread | Trigger anywhere; root-party walk-back authority |

If dashboard comms encoded the decomposition DAG explicitly, each message would carry:
- The node it triggers on
- The claim it disputes
- The visited-ancestor trail (after walk-back resolves)

This would let us **structurally trace** the cascade replanning across sessions, instead of relying on prose narratives. The dashboard's existing message log + work-item graph becomes the decomposition DAG.

---

## §6. Open questions for iteration

1. **Coefficient semantics** — confirmed by operator as balance-enforcers (not effort weights). But how strictly to enforce sum-to-1? Approximate (parties just declare shares ad-hoc) or hard (rational arithmetic checked)? Suggest hard for `StructuralLens` witnesses, approximate for `ProseAttestation`.
2. **Claim equivalence** — when do two parties' interpretations of a claim count as "agreeing"? Same text suffices? Same evidence-for-match required? In gunbc terms, this is whether `Claim.text` is opaque-string (rejected per `feedback_opaque_strings_attract_heuristics`) or a structural composition over typed primitives.
3. **Walk-back termination at root** — what does "root-asker" mean operationally? For our project: operator (briansrls). For substrate-level decompositions: the relevant Mgr / Director. Need to confirm the hierarchy explicitly.
4. **Cost-of-rebalance** — the user noted that walk-back forces re-decomposition cascades. How does the algebra encode the *cost* of a rebalance? Cost lens analog: every `WalkBackEvent` has a cost = `Σ work_undone_below(stable_ancestor)`. This could become a `RebalanceCost` dimension parallel to `SymbolicCost`.
5. **Implementation surface** — start as prose discipline in CLAUDE.md / a `feedback_decomposition_algebra` memory? Or skip straight to a `.dag` model in `dsl/std/`? Suggest prose discipline first to validate the procedure on N=3-5 real cases (PR #2745 + EmissionTarget rename + others); then formalize as `.dag` lens.

---

## §7. Next step

Validate the procedure on more recent cases:
- ✓ PR #2745 misread (§4 above)
- TODO: EmissionTarget rename cascade — walk-back from operator BLOCKING at PR #2749 :666 (post-merge name collision) through SELF_HOSTING.md authority back to substrate-shape ratification
- TODO: PR #2744 cascade BLOCKINGs (PythonShim asymmetry; dissolution sketch; absent option) — each is a walk-back trigger; algebra would have caught each at brief-authoring time

If the procedure surfaces the gap structurally in all 3-5 cases, formalize as `.dag` lens + integrate into dashboard comms.
