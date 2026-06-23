# Resolver type-name-collision wall (§5 construction wall) — model-before-implement

Status: **MODEL ONLY.** No resolver edit lands without parent (bright-stag) checkpoint-sign
AND the std de-fork (sunny-cat-114 lane) reporting clean. This document is the model brought
to that checkpoint. Authoring the guard/witness is in-scope; flipping the live fail-closed
gate is operator-gated and Route-C-sequenced.

Owner: jolly-ant-231 (deepest resolver context from the A1 diagnosis). Capstone of the std
de-fork lane.

---

## 1. The hole this wall closes (grounding)

The A1 (#5615) incident — `type 'EqualsClaim' not found in scope` on
`src/v2/workflow/affected_set_floor_runner_test.dag` — was a **fail-OPEN in the resolver**
(DESIGN §5): a wrong type resolution passed silently as a different, plausible binding.

Mechanism (proven by execution during the diagnosis; receipt below):

- The per-module type scope (`TypeEnv.bindings`) is a `HashMap<i64, Rc<TypeBinding>>` keyed by
  the **interned id of the UNQUALIFIED type name** — the intern table is global, so two
  module-level types sharing a short name (`v2.std.verification.TestClaim` the Disj coproduct
  vs `std.verification.TestClaim` the Conj record) hash to the **same key**.
- `import_bindings` is assembled by folding each imported module's bindings with a blind
  `v1_rt::rc_map_merge` (`v1_compiler_infer.rs` ~11799 and the duplicate ~12393). **Map merge
  overwrites on duplicate key — last writer wins, no diagnostic.** One authority silently
  shadows the other.
- Because variant-constructor registration (`build_module_context`'s `variant_fold`, ~12894)
  only fires for `Disj` bindings, when the **record** won the `TestClaim` slot the coproduct's
  constructors (`EqualsClaim`, `StructuralEqualsClaim`, …) were never registered → "not found
  in scope". A1 was an innocent trigger: it merely grew the test's import-closure to include
  `std.verification(dsl)` (via `gunbc.ci_failure_class → extdeps.cache.sccache →
  std.cache_interface → std.verification`).

The existing **`VariantCollision`** guard (`v1_std_core.rs:420`,
`VariantCollision { variant, enum1, enum2, span }`; emitted in `variant_fold` ~12908) already
fails-closed on two imported coproducts sharing a **variant** name. It does **not** cover the
level above: two modules contributing the same **type/enum name** to one closure. The wall is
that guard lifted one level up — the §5 principle of making the bad state *unwritable* rather
than letting it resolve to a plausible wrong binding.

`#5640` (the `TestClaim → AssertionClaim` rename) dissolved the single instance. The wall
makes the **class** unwritable so the next closure-expanding PR cannot re-arm it.

Decidability: a closure exporting two distinct authorities under one unqualified name is a
**decidable** condition (finite set of `(name_id, declaring_file)` pairs), so this is a §5
*wall now*, not a forever-ratchet. "never" is honest here.

---

## 2. Mechanism (the model)

Replace the blind `rc_map_merge` of type bindings, at **both** import-binding assembly sites
(`v1_compiler_infer.rs` ~11761 and ~12355 — themselves a §2 duplication this work should
dissolve into **one** shared helper), with a **collision-checking merge**:

```
merge_type_bindings(acc, incoming, policy) -> { bindings, collisions }
  for each (name_id, incoming_binding) in incoming:
    case acc.get(name_id):
      None            -> insert
      Some(existing)  ->
        if same_authority(existing, incoming):        # re-export of ONE authority
            keep existing (no flag)
        else if policy.flags(existing, incoming):      # TWO authorities, one name
            emit TypeNameCollision{ name, module_a, module_b, span }
            keep existing (deterministic) — fail-closed downstream
        else:
            keep existing (policy excused it)
```

Where:

- `same_authority(a, b)` ≡ `a.resolved.span.file == b.resolved.span.file` (one declaring file
  reached via multiple import paths is the SAME authority — a benign re-export, must NOT
  flag). `TypeBinding` is `{ name, resolved: Rc<Node>, provenance }` and `Node.span.file`
  carries the declaring file, so two-authorities-vs-one-re-export is distinguishable today —
  **no new field required.**
- `TypeNameCollision` is a new `CompilerDiagnostic` variant modeled exactly on
  `VariantCollision`: `{ name: String, module_a: String, module_b: String, span }`, classified
  **blocking** (`is_error_diagnostic` true), so it fails the resolve/typecheck closed with a
  located, typed message — the §5 "loud error, never a warning".
- Determinism: on collision keep the **first-seen** binding (stable under the existing
  dep-order traversal) so the error set is reproducible regardless of which authority "would
  have won" the old overwrite.

This is one mechanism; §3 (the policy) and §4 (activation) are knobs on it, not separate
machinery.

---

## 3. KEY DECISION (parent's Q2): which collisions fail closed?

**One predicate, one boolean knob** — `policy.flags(existing, incoming)`:

- **flag-ANY** (`require_structural_divergence = false`):
  `flags = true` for any two distinct authorities (different declaring files) under one name.
  Even a **byte/structurally-identical mirror** flags — because it is still *two authorities
  for one concept*, the §3 single-authority violation, just benign-looking. It remains a
  writable re-fork surface: nothing stops the two copies drifting apart later (which is exactly
  how A1's landmine formed).

- **flag-DIVERGENT** (`require_structural_divergence = true`):
  `flags = structural_inequality(existing.resolved, incoming.resolved)`.
  Fires only when the two definitions actually differ (connective and/or children) — like the
  Disj-coproduct vs Conj-record case. True mirrors are **excused**.

`structural_inequality` is a normalized structural compare of the two resolved type Nodes
(connective + children names + field types), modulo span/occurrence — the same shape the
emitter's coercion homomorphism check already walks; no new comparison primitive needed.

### Trade-off (parent rules on the real distribution)

| | flag-ANY | flag-DIVERGENT |
|---|---|---|
| Catches the A1 class (divergent shadow) | yes | yes |
| Catches a *future* mirror that later drifts | yes (at fork time) | no (only after drift, when a closure re-imports both) |
| De-fork scope it demands | **all** same-name pairs resolved: divergent → rename-apart, mirror → **merge to one authority** | only divergent pairs resolved; mirrors may stay |
| False-positive risk | a legitimately-duplicated mirror that *cannot* be merged yet would block until merged | a divergent pair that *looks* similar but isn't structurally-equal still correctly fires |
| §3 fidelity | full (one name ⇒ one authority, tree-wide) | partial (tolerates two authorities iff currently identical) |

**Parent leans flag-ANY** (a mirror is still two authorities = the §3 violation). I concur on
§3 grounds, *conditional on* the de-fork audit: flag-ANY is correct **iff** every true-mirror
among the 9 basenames can be **merged to a single authority** (not just renamed-apart). If the
audit finds a mirror that legitimately *cannot* be unified yet (e.g. a staged migration that
must keep two homes transiently), flag-ANY would block it and flag-DIVERGENT is the honest
interim. **The decision waits on sunny-cat-114's per-basename audit** of
`algebra · coercion · effects · float · integer · logic · nat · node · verification`:
each = divergent-concept (rename-apart) or true-mirror (merge). Design is policy-agnostic so
parent rules on the measured distribution, not from the chair.

(Coordination note: `sunny-cat-114` was not yet reachable as a session at authoring time —
flagged to manager; this section folds in the audit the moment it lands.)

---

## 4. ACTIVATION / DORMANCY (parent's Q3): **lands-already-green** (preferred)

Two options:

- **Staged-enable** (guard lands dormant, flips fail-closed later): carries **dormant-guard
  state** — a loaded mechanism left unwired. DESIGN §5/§6 names this an anti-pattern: an inert
  guard is "coverage by illusion", a latent lie until the flip, and adds a flip-event to
  sequence and a window where the guard exists but does not gate.

- **Lands-already-green** (preferred): the de-fork lane first removes **every** collision the
  chosen policy flags from the tree; *then* the guard lands **already fail-closed and green**.
  No dormant state, no flip event. The guard's own green landing is **self-certifying**: it is
  green only if zero flagged collisions remain, so merging it *proves* the tree is
  collision-free under the policy. This is the §5 "wall lands live or not at all".

Sequencing (hard dependency): **de-fork clean (per chosen policy) → guard lands fail-closed,
green, in one PR.** Concretely, the guard PR is mergeable iff the full floor corpus resolves
clean with the guard active — which holds iff the de-fork eliminated all flagged same-name
pairs. Under flag-ANY that means all 9 basenames de-forked (renamed-apart **or**
merged-to-one-authority); under flag-DIVERGENT only the divergent subset.

This makes the wall the **capstone** of the de-fork lane, exactly as framed: the de-fork is not
"cleanup that enables a later guard" — the guard *is* the de-fork's proof of completion.

---

## 5. Proof plan (model-only; how it is proven by execution when authored)

Per DESIGN §5 ("done" = a real consumer green by execution + a discriminating input that goes
red when the behavior is wrong):

- **GREEN-by-execution:** with the de-fork complete, run the exact floor closure
  (`claim_executor --source-root dsl --source-root src/v2`, both orders) — full corpus resolves
  clean **with the guard active**. (Baseline receipt already in hand from #1: post-rename,
  `affected_set_floor_runner_test` PASSes both orders, 56 modules.)
- **Discriminating RED (the falsifier):** re-introduce a single synthetic same-name pair (e.g.
  a throwaway `std.fixture.dup_typename` exporting a `TestClaim`) into a test's closure and
  assert the guard emits `TypeNameCollision` and fails closed — and that *removing* it returns
  green. Under flag-DIVERGENT, add the dual control: a structurally-identical mirror does **not**
  fire while a divergent pair does.
- **No-regression control:** a type legitimately re-exported through multiple import paths
  (one declaring file, many hops) must stay green — proves `same_authority` (span.file
  identity) correctly excuses re-exports and the wall is not over-broad.

---

## 6. Scope guard / what this is NOT

- **No resolver edit lands** here — model only until checkpoint-sign + de-fork clean.
- Does **not** touch the `module std.verification` name-fork or the dsl↔v2 std module
  reorganization (Route C) — orthogonal, operator-strategic. The wall keys on type-name
  identity within a closure and is compatible with whatever Route C decides.
- Mirrors the existing `VariantCollision` precedent; introduces no new resolver concept beyond
  lifting that guard one level and dissolving the duplicate import-binding folds into one
  checked-merge helper (a §2 win bundled in).

## 7. Open items

- sunny-cat-114 per-basename divergent/mirror audit → settles §3 policy + §4 de-fork scope.
- Confirm both import-binding assembly sites (~11761, ~12355) are the only type-binding merge
  loci (the `std.types`-special-cased branch folds key-by-key already and is naturally covered
  by the same helper).
- Parent ruling on flag-ANY vs flag-DIVERGENT against the measured distribution.
