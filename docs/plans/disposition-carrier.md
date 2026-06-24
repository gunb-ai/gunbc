# Plan — the `Disposition` carrier (confront the skipped modeling decisions)

**Status:** planning tracker · **DESIGN.md + carriers are authority** (§6 no parallel ledger). Linked from `ROADMAP.md §0`. The first instance of a standing practice: now that `.dag` is mature **and lens exists** (it did not when the `🟡` comments were written), every modeling ambiguity we used to skip gets **confronted** — resolved into construction or a justified-terminal lens, not a comment.

**Verified against the live tree 2026-06-21.** Receipts below; re-check before acting.

## 0. The problem — disposition lives in comments, unreadable by lens

A coproduct's *disposition* — is it closed **for a reason** (`Terminal`) or a **stand-in awaiting a construction** (`Scaffold`) — is recorded today only as freeform comments:

- `// 🟢 TERMINAL — closed by the REST protocol grammar …` (`extdeps/object_storage/object_storage.dag:52`)
- `// 🟡 gated — feature:… — bind node://… ` + `// dissolve-on-arrival: project this corpus directly from the service decl` (`extdeps/*/mock_corpus.dag`)

A lens is a pure reader over the **Node tree**; comments are not Nodes, so **nothing can read these** (same reason `render.dag` is "not witnessable"). The untagged/indeterminate state — "is this closed for a reason or just unfinished?" — is the fail-open state, and it is arbitrary (DESIGN §1: arbitrary = convention standing where necessity was available).

There is also no typed carrier: `Terminal`/`Scaffold` are text only; `src/v2/lens/registry.dag` lists lenses (`LensRegistryEntryV0`) for **9 of ~35** files with no disposition field.

### 0a. Post-#5579 reframe — the wall moved the ball halfway *and* grew the surface

The comment-ban wall (#5579, `//` is now a parse error) changed the shape of this problem, it did not solve it. The marks that used to live in comments were forced **out of comments and into `data: String` rows** — `bytes_seam`, `unit_must_run_staged_note`, the Anthropic closure rows, the budget-tree leaves, `consumed_input_closure_concept_note` / `_convergence_candidate` / `_slice1_status` (#5605). So the problem statement is no longer "disposition lives in **comments** (invisible to a lens)" but "disposition lives in `data: String` **rows** — **visible** to a lens as a Node, but still **opaque prose** whose *meaning* a lens cannot read." Net: the wall made the marks Nodes (a lens can now *see* them) but the §3-migrate surface **grew** to include the whole post-wall `data:String` fleet on top of the original `🟡` extdeps comments. That **strengthens** this carrier's case — the displaced cost it removes is now bigger and concrete, with named instances (§6: denominate the benefit). It does not change the carrier: `data:String` prose → typed `Disposition` is the same move, with a larger, already-Node-shaped input set.

This is the chosen first proof-by-use region: **slice-1** (operator GO 2026-06-23, work-item `adhoc-0a633bef-bb9`, `fierce-crane-13` under `neat-dove-397`) models `Disposition` + `ConstructionMechanism` in `std` (a DESIGN-named load-bearing carrier → **checkpoint** to `bright-stag-194` before any row migrates), migrates **this** `data:String` fleet region-by-region (each region red-on-revert so the lens is never inert mid-migration), and lands the fail-closed redundancy lens green-by-execution with a discriminating control. The substrate-mandatory **#1** end-state stays the named goal, not this slice — slice-1 *proves the binary taxonomy by use before cementing it* (see §2a).

## 1. The carrier — one concept, two carriers (§2 horizontal, single authority)

Lens-lifecycle tags and coproduct dissolve-markers are the **same concept**. One typed carrier:

```
type Disposition =
  | Terminal { reason: String }                      // legitimately closed/lens-forever (the unstructurable: complexity / cost / necessity)
  | Scaffold { dissolves_to: ConstructionMechanism, bind: NodeRef }   // stand-in until that construction lands
```

Applied to **both** a coproduct `type` decl and a lens registry entry. Single authority — not two marking schemes. (`dissolves_to` is itself modeled, not a string: the mechanism that will make the scaffold dead code — e.g. `SingleAuthority`, `RealizationDispatch`, `SubstrateMandatoryTag`.)

**Carrier note (drift to reconcile):** `bind: NodeRef` above is the *resolved, symbolic* end-state. The landed `std/disposition.dag` ships `bind: DeclarationRef` — a stringly `{ module_path, decl_name, field }` pair — as a gated-interim shim (the resolution it needs is blocked on gunbc#5364). That stringly form is itself a §3 anemia the carrier exists to dissolve; tracked as its own region in **§7**.

## 2. The decision — #1 vs #2 vs middle (proactive vs lens-enforced)

| approach | what | verdict |
| --- | --- | --- |
| **#1 proactive** | can't *define* a coproduct untagged (substrate wellformedness) | the end-state (purest construction), but a **load-bearing §4 substrate change** → escalate; **flag-day** whole-tree migration; **derived** coproducts need disposition derived, not authored |
| **#2 lens-only** | optional tag, a lens checks presence | validation; the weak form |
| **middle (CHOSEN)** | construction-*capable* carrier + lens enforces presence **selectively, ratcheting coverage outward** | rolls out region-by-region; discovers the derived-coproduct cases; proves the taxonomy **by use** before cementing into the grammar |

**Chosen: middle now, #1 as the named end-state.** The lens is itself a `Scaffold{dissolves_to: SubstrateMandatoryTag}` — when coverage = whole tree **and** the substrate can require the tag at definition, the lens is dead code and **dissolves**. The enforcement mechanism is disposed of *by* the discipline (same shape as the numeric-tower guard going dead). Jumping straight to #1 is rejected only on sequencing (substrate change + flag-day + unproven taxonomy), not on principle.

### 2a. Taxonomy stress-test — the hypothesis slice-1 must try to falsify

The first real test input is #5605's `ConsumedInputClosure`, which appears to wear **three** dispositions at once: the concept is sound (`_concept_note`), the impl is a scaffold (`_slice1_status`), and a convergence is pending (`_convergence_candidate`). This *looks* like the binary `Terminal | Scaffold` is too narrow and needs a soundness/completeness/convergence **axis split**. It is not — and that is the ruling slice-1 carries in as a hypothesis to **falsify**, not a blank:

- It is **N marks per carrier, not N axes per `Disposition`.** Those three are three *distinct* `data:String` rows, each cleanly one disposition: `_concept_note` → `Terminal{reason}` (consume-input vs produce-artifact is a permanent §3 distinction — the concept stays); `_convergence_candidate` → `Scaffold{dissolves_to: the one selection authority all consumers reach}`; `_slice1_status` → `Scaffold{dissolves_to: per-unit selection}`. (Convergence-pending *is* a `Scaffold` — "these N authorities should become 1" is a §3 redundancy that dissolves when the merge lands — not a third axis.)
- Splitting `Disposition` into axes would **grow the concept** to model what multiple-marks-per-carrier already expresses (§2: net concepts must not grow by re-invention) — a failed decomposition.
- **The discriminating test:** can every mark be assigned *exactly one* `Disposition` without losing information? If yes, the binary holds and the multiplicity is just per-carrier mark count. The taxonomy needs the split **only** if a *single, indivisible* mark genuinely needs two dispositions at once (sound-concept AND scaffold-impl fused in one row that cannot be split into two marks) — and green-by-execution would then show it. Until a mark fails that test, the binary is the cheaper, more grounded answer.

## 3. What is actually enforceable (construction vs lens vs retro)

- **Presence of a disposition** — *construction.* Non-optional field on the carrier ⇒ you cannot author the carrier without committing to `Terminal`/`Scaffold`. The type enforces it; **no meta-lens** (which would itself be validation, the rule §0 just adopted).
- **Redundancy not cleaned up** — *fail-closed lens.* RED when a `Scaffold{dissolves_to: X}` coexists with an already-present `X` (scaffold + its named successor both exist = §2 parallel-representation debt). The one hard gate.
- **Correctness of `Terminal` vs `Scaffold`** — *retro, judgment, NOT a gate.* Classifying "this property is structuralizable into a single authority" is the synthesis-feasibility limit / leaf-decomposition-diagnosis (both operator-parked open threads). Mis-tagging is caught by a periodic **advisory report** (list every `Scaffold`, flag ones whose `dissolves_to` already exists, surface `Terminal`s for re-justification), never RED.

## 4. The deepest form, where it applies — dissolve the marker entirely

For the `mock_corpus` scaffolds (`dissolve-on-arrival: project from the service decl`), scaffold and successor are two **realizations of one corpus interface** — so the ideal is not marker+lens at all but the §2 Realization pattern: dispatch selects the projected handler when present → the hand-authored arm is **dead code**. No marker, no lens. The `Disposition`+redundancy-lens is the **residue** for cases that are *not* a single interface.

## 5. Sequencing

1. Model `Disposition` (+ `ConstructionMechanism`) as a typed carrier in std.
2. Add a non-optional disposition field to `LensRegistryEntryV0`; **complete the registry to all ~35 lenses** (or move disposition onto each lens module, discovered tree-wide like `*_test.dag`) — this roster gap is the prerequisite to making the field non-optional. Each existing lens resolves to `Terminal{reason}` (complexity/cost/necessity) or `Scaffold{dissolves_to}` (everything §0 tier-1).
3. Migrate the `🟡` extdeps comments → typed `Disposition` fields, region by region.
4. Land the redundancy lens (§3) over all dispositions; enroll it on the first region.
5. Ratchet the enforced region outward PR by PR — next region is the **`_contracts` coproduct wire-contracts** (§6: 20 contracts, ungated via `enumerate_coproduct_decls()`).
6. Add the decl-ref **resolution lens** (§7 step 1) — every `Scaffold.bind` / `CoproductWireContract.coproduct` must resolve to an enumerated coproduct decl (ungated); then **ground `DeclarationRef` → resolved `NodeRef`** once gunbc#5364 lands (§7 step 2).
7. **Escalate** the substrate change (#1: can't-define-untagged) when coverage = whole tree and the taxonomy is proven; then the lens dissolves.

## 6. The total migration surface (what "cleaned up" means in full)

"Total migration" = every place a disposition lives as prose **or** as an untyped/stringly carrier becomes a typed, resolvable `Disposition`. Five regions, ordered by what the substrate can already enumerate (buildable-now first, gated last):

| region | count / where | enumerable now? | status |
| --- | --- | --- | --- |
| **post-#5579 `data:String` fleet** | fleet / runner / ci / entropy / bmc marks | yes (already Node rows) | in progress — #5746 (9 → `Scaffold`), #5740 (4 → `Terminal`) |
| **`_contracts` coproduct wire-contracts** | 20 across 9 files (docker, github×6, llm×2, systemd) | **yes** — `enumerate_coproduct_decls()` is structural / landed | **0/20 tagged** — the recommended next ratchet region |
| **lens registry entries** | ~26 of ~35 untagged (`LensRegistryEntryV0`) | yes (registry roster) | §5 step 2 — the registry-completion prerequisite |
| **`🟡` extdeps comments** | `mock_corpus.dag` dissolve-on-arrival | §4 Realization dissolves these (no tag at all) | deepest form — dispatch makes the hand-arm dead code |
| **stringly `bind` itself (§7)** | every `Scaffold.bind` + every `CoproductWireContract.coproduct` | module→path now (#5675); field-granular **gated on gunbc#5364** | `DeclarationRef` → resolved `NodeRef` — see §7 |

The `_contracts` region is the **right next slice after the `data:String` fleet**: the carrier is already typed (`CoproductWireContract`) and the substrate can already enumerate every coproduct (`v2.std.concept_index.enumerate_coproduct_decls()`, landed structural per the testgen oracle), so the presence/redundancy lens can run over real coproduct decls **today with no #5364 dependency**. Each of the 20 contracts gets a `Disposition` — almost all `Terminal { reason: closed by the upstream wire-protocol grammar }`, since a service's enum *is* its closed wire vocabulary — and the presence lens ratchets to cover the `_contracts` region. (Operator's question — "is there a requirement to tag coproducts in `_contracts` with a dissolution tag?" — answer: **not enforced yet**; that requirement *is* the #1 `SubstrateMandatoryTag` end-state, reached here region-by-region with `_contracts` as the next region, not by flag-day.)

## 7. Symbolic binds — `DeclarationRef` (stringly) → resolved `NodeRef`

**Carrier drift to reconcile:** §1 specifies `Scaffold { dissolves_to, bind: NodeRef }` (a *resolved, symbolic* reference). The landed `std/disposition.dag` shipped `bind: DeclarationRef` — `{ module_path: NonEmptyStr, decl_name: NonEmptyStr, field: DeclField }`, i.e. **two raw strings**. That is itself the §3 anemia / nickname problem the carrier exists to remove: `decl_name: "cited_operating_system_surface"` is a *string that names a decl*, not a reference *to* it, so nothing verifies the decl exists — a typo, a rename, or a deletion leaves the bind silently dangling (fail-open). The same stringly pair is already load-bearing in all 20 `CoproductWireContract.coproduct` fields.

**Why it shipped stringly (the gate):** resolving `module_path` + `decl_name` (+ `field`) to the actual declaration Node needs field-granular decl enumeration over the self-host compile graph, **gated on gunbc#5364**. What exists today: `module_declaration_facts` (#5675, module→path) and `enumerate_coproduct_decls()` (type-decl granular). So `DeclarationRef` is an honest *transitional shim* — but, until now, an **untracked** one.

**The enforcement gap the operator flagged** ("a lens to enforce that?"): there is **no lens today that resolves a `DeclarationRef`** — neither that the named decl exists, nor that wire-contract / disposition decls live where they should. `module_graph.dag` reads imports, not decl-ref targets. So a `DeclarationRef` is currently a §5 *wall-after-grounding*: decidable (does `module_path::decl_name` resolve to a real decl?) but waiting on its single authority (the #5364 enumeration).

1. **Now (ungated):** add a **resolution lens** keyed on `enumerate_coproduct_decls()` — every `CoproductWireContract.coproduct` and every coproduct-targeting `Scaffold.bind` must resolve to an enumerated coproduct decl; **RED on a dangling `decl_name`**. This already covers the 20 `_contracts` (all target coproducts) and the coproduct binds, with no #5364 dependency. It is the discriminating consumer that makes the `_contracts` migration non-inert.
2. **Gated on gunbc#5364:** widen the resolution lens to *all* decls and to field-granularity (`NamedField`), then **ground `DeclarationRef` → `NodeRef`**: `bind` / `coproduct` carries a *resolved* node reference, and an unresolvable reference is **unwritable** (construction, not validation — §5). At that point `DeclarationRef`'s raw-string form dissolves into the resolved carrier.

This recovers §1's original `NodeRef` intent: stringly `DeclarationRef` is the gated-interim, resolved `NodeRef` is the end-state, and the resolution lens is the ratchet between them — itself a `Scaffold { dissolves_to: SubstrateMandatoryTag }` that dies when the substrate resolves references at definition.

## Dissolution trigger (DESIGN §6)

Delete this doc when `Disposition` is a substrate-mandatory field on every coproduct (#1 reached, the ratcheting lens is dead code) and the redundancy lens is whole-tree — at which point an untagged or not-cleaned-up coproduct is unwritable and this tracker is redundant.
