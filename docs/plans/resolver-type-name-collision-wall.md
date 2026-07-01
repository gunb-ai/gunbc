# Resolver type-name-collision wall (§5 construction wall) — model-before-implement

Status: **MODEL — SIGNED (parent bright-stag, 2026-06-23).** The model (mechanism + Q2 policy + activation) is signed; **no resolver edit lands** until the §4 grounding de-fork is clean — the resolver guard is a separate, operator-gated future PR that lands on the de-forked tree. This document is the signed model; §3/§4 record the final ruling.

Owner: jolly-ant-231 (deepest resolver context from the A1 diagnosis). Capstone of the std de-fork lane. Q2 evidence: `docs/plans/dsl-v2-defork-audit.md` §2A (executed distribution).

---

## 1. The hole this wall closes (grounding)

The A1 (#5615) incident — `type 'EqualsClaim' not found in scope` on `src/v2/workflow/affected_set_floor_runner_test.dag` — was a **fail-OPEN in the resolver** (DESIGN §5): a wrong type resolution passed silently as a different, plausible binding.

Mechanism (proven by execution during the diagnosis; receipt below):

- The per-module type scope (`TypeEnv.bindings`) is a `HashMap<i64, Rc<TypeBinding>>` keyed by the **interned id of the UNQUALIFIED type name** — the intern table is global, so two module-level types sharing a short name (`v2.std.verification.TestClaim` the Disj coproduct vs `std.verification.TestClaim` the Conj record) hash to the **same key**.
- `import_bindings` is assembled by folding each imported module's bindings with a blind `v1_rt::rc_map_merge` (`v1_compiler_infer.rs` ~11799 and the duplicate ~12393). **Map merge overwrites on duplicate key — last writer wins, no diagnostic.** One authority silently shadows the other.
- Because variant-constructor registration (`build_module_context`'s `variant_fold`, ~12894) only fires for `Disj` bindings, when the **record** won the `TestClaim` slot the coproduct's constructors (`EqualsClaim`, `StructuralEqualsClaim`, …) were never registered → "not found in scope". A1 was an innocent trigger: it merely grew the test's import-closure to include `std.verification(dsl)` (via `gunbc.ci_failure_class → extdeps.cache.sccache → std.cache_interface → std.verification`).

The existing **`VariantCollision`** guard (`v1_std_core.rs:420`, `VariantCollision { variant, enum1, enum2, span }`; emitted in `variant_fold` ~12908) already fails-closed on two imported coproducts sharing a **variant** name. It does **not** cover the level above: two modules contributing the same **type/enum name** to one closure. The wall is that guard lifted one level up — the §5 principle of making the bad state *unwritable* rather than letting it resolve to a plausible wrong binding.

`#5640` (the `TestClaim → AssertionClaim` rename) dissolved the single instance. The wall makes the **class** unwritable so the next closure-expanding PR cannot re-arm it.

Decidability: a closure exporting two distinct authorities under one unqualified name is a **decidable** condition (finite set of `(name_id, declaring_file)` pairs), so this is a §5 *wall now*, not a forever-ratchet. "never" is honest here.

---

## 2. Mechanism (the model)

Replace the blind `rc_map_merge` of type bindings, at **both** import-binding assembly sites (`v1_compiler_infer.rs` ~11761 and ~12355 — themselves a §2 duplication this work should dissolve into **one** shared helper), with a **collision-checking merge**:

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

- `same_authority(a, b)` ≡ `a.resolved.span.file == b.resolved.span.file` (one declaring file reached via multiple import paths is the SAME authority — a benign re-export, must NOT flag). `TypeBinding` is `{ name, resolved: Rc<Node>, provenance }` and `Node.span.file` carries the declaring file, so two-authorities-vs-one-re-export is distinguishable today — **no new field required.**
- `TypeNameCollision` is a new `CompilerDiagnostic` variant modeled exactly on `VariantCollision`: `{ name: String, module_a: String, module_b: String, span }`, classified **blocking** (`is_error_diagnostic` true), so it fails the resolve/typecheck closed with a located, typed message — the §5 "loud error, never a warning".
- Determinism: on collision keep the **first-seen** binding (stable under the existing dep-order traversal) so the error set is reproducible regardless of which authority "would have won" the old overwrite.

This is one mechanism; §3 (the policy) and §4 (activation) are knobs on it, not separate machinery.

---

## 3. KEY DECISION (parent's Q2): which collisions fail closed?

**One predicate, one boolean knob** — `policy.flags(existing, incoming)`:

- **flag-ANY** (`require_structural_divergence = false`): `flags = true` for any two distinct authorities (different declaring files) under one name. Even a **byte/structurally-identical mirror** flags — because it is still *two authorities for one concept*, the §3 single-authority violation, just benign-looking. It remains a writable re-fork surface: nothing stops the two copies drifting apart later (which is exactly how A1's landmine formed).
- **flag-DIVERGENT** (`require_structural_divergence = true`): `flags = structural_inequality(existing.resolved, incoming.resolved)`. Fires only when the two definitions actually differ (connective and/or children) — like the Disj-coproduct vs Conj-record case. True mirrors are **excused**.

`structural_inequality` is a normalized structural compare of the two resolved type Nodes (connective + children names + field types), modulo span/occurrence — the same shape the emitter's coercion homomorphism check already walks; no new comparison primitive needed.

### Trade-off (parent rules on the real distribution)

|  | flag-ANY | flag-DIVERGENT |
| --- | --- | --- |
| Catches the A1 class (divergent shadow) | yes | yes |
| Catches a *future* mirror that later drifts | yes (at fork time) | no (only after drift, when a closure re-imports both) |
| De-fork scope it demands | **all** same-name pairs resolved: divergent → rename-apart, mirror → **merge to one authority** | only divergent pairs resolved; mirrors may stay |
| False-positive risk | a legitimately-duplicated mirror that *cannot* be merged yet would block until merged | a divergent pair that *looks* similar but isn't structurally-equal still correctly fires |
| §3 fidelity | full (one name ⇒ one authority, tree-wide) | partial (tolerates two authorities iff currently identical) |

### RULED — `flag-ANY` (parent bright-stag, signed 2026-06-23)

**`require_structural_divergence = FALSE`.** Settled against the measured distribution (the executed per-basename audit in `docs/plans/dsl-v2-defork-audit.md` §2A — `structural_inequality` run over all 9 std basename pairs + a 351-entry floor-closure reachability BFS). The audit confirmed the cant-unify-yet set (basenames needing a milestone *later than* the grounding de-fork) is **{node, coercion} exactly** — no third stuck mirror — so flag-ANY does not block on an un-unifiable pair.

Two consequences parent reconfirmed:

- **flag-ANY FIRES on byte-identical mirror shared type-names too** (the audit's 3 in `algebra`, 3 in `integer`, 1 in `float`), not only on differing ones. This is **intended**: a byte-identical mirror is still *two homes for one type* = the §3 single-authority violation and a writable re-fork surface. So the de-fork must **merge even the mirrors** (delete the v2 copy, repoint imports), and flag-ANY is exactly what forces that. flag-DIVERGENT would *tolerate* the byte-mirrors — a standing §3 hole — so it is strictly worse here (it also still fires on every grounding, sparing nothing). Mirrors are therefore **resolved by the de-fork, not exempted**.
- **The exemption roster carries NO `{node, coercion}` entry — DROPPED as vacuous.** node and coercion (and `logic`, and post-#5640 `verification`) share **zero** type names — only the *basename* — so the type-name guard **never fires** on them. A roster entry for a collision the mechanism cannot produce is a §5 inert scaffold (a dead line). Their collision is a module-**basename** rename on the Route-C / v1-delete surface — a *different* mechanism, still tracked (audit category-(c), `dissolve-on = v1-delete`), not by this guard. The guard's roster is therefore **deliberately empty** — recorded here so its emptiness reads as a decision, not an omission. (See §6.)

---

## 4. ACTIVATION / DORMANCY (parent's Q3): **lands-already-green** (preferred)

Two options:

- **Staged-enable** (guard lands dormant, flips fail-closed later): carries **dormant-guard state** — a loaded mechanism left unwired. DESIGN §5/§6 names this an anti-pattern: an inert guard is "coverage by illusion", a latent lie until the flip, and adds a flip-event to sequence and a window where the guard exists but does not gate.
- **Lands-already-green** (preferred): the de-fork lane first removes **every** collision the chosen policy flags from the tree; *then* the guard lands **already fail-closed and green**. No dormant state, no flip event. The guard's own green landing is **self-certifying**: it is green only if zero flagged collisions remain, so merging it *proves* the tree is collision-free under the policy. This is the §5 "wall lands live or not at all".

Sequencing (hard dependency, RULED): **the grounding de-fork of the shared-type-name set `{algebra, nat, effects, float, integer}` → guard lands flag-ANY, fail-closed, green, in one PR.** Each of those five is merged to a single authority (delete the v2 copy / repoint, or the operator grounding-unification design where the bodies diverge) — mirrors included (§3). The set is fronted by the **LIVE pair `{algebra (75 floor entries), nat (4)}`** (their `std.<b>` / `v2.std.<b>` co-occur in real floor closures today and silently fail-open — benign only because they shadow record-with-record, not the coproduct-variant-drop that broke `verification` under A1) and completed by the **latent `{effects, float, integer}`** (no co-occurring closure today, but re-arm risks — exactly how `verification` went latent→LIVE under A1's closure expansion). `verification` already cleared (#5640); `{node, coercion, logic}` are off this surface (zero shared type-names).

The guard PR is mergeable iff the full floor corpus resolves clean **with the guard active** — which holds iff that five-basename de-fork is complete. Its green landing therefore **self-certifies** the shared-type-name surface is collision-free: the wall is the **capstone** of the de-fork lane, correctly located. No dormant-guard state, no flip event — lands-already-green.

---

## 5. Proof plan (model-only; how it is proven by execution when authored)

Per DESIGN §5 ("done" = a real consumer green by execution + a discriminating input that goes red when the behavior is wrong):

- **GREEN-by-execution:** with the de-fork complete, run the exact floor closure (`claim_executor --source-root dsl --source-root src/v2`, both orders) — full corpus resolves clean **with the guard active**. (Baseline receipt already in hand from #1: post-rename, `affected_set_floor_runner_test` PASSes both orders, 56 modules.)
- **Discriminating RED (the falsifier):** re-introduce a single synthetic same-name pair into a test's closure and assert the guard emits `TypeNameCollision` and fails closed — and that *removing* it returns green. Run it **twice** to pin the flag-ANY semantics: once with a **divergent** pair (different bodies) and once with a **byte-identical mirror** pair — *both* must fire (flag-ANY excuses neither). A flag-DIVERGENT build would let the mirror through; that it does **not** is the §3-fidelity control.
- **No-regression control:** a type legitimately re-exported through multiple import paths (one declaring file, many hops) must stay green — proves `same_authority` (span.file identity) correctly excuses re-exports and the wall is not over-broad.

---

## 6. Scope guard / what this is NOT

- **No resolver edit lands** here — model only until checkpoint-sign + de-fork clean.
- Does **not** touch the dsl↔v2 std module-**basename** reorganization (Route C / v1-delete) — orthogonal, operator-strategic. The wall keys on shared **type-name** identity within a closure, so the zero-shared-type-name basenames `{node, coercion, logic, verification}` are off its surface entirely (this is why the roster carries no `{node, coercion}` entry, §3). Their basename de-fork stays tracked in the audit's category-(c) (`dissolve-on = v1-delete`), a different mechanism. The wall is compatible with whatever Route C decides.
- Mirrors the existing `VariantCollision` precedent; introduces no new resolver concept beyond lifting that guard one level and dissolving the duplicate import-binding folds into one checked-merge helper (a §2 win bundled in).

## 7. Open items

- **RESOLVED** — Q2 policy: `flag-ANY` signed (§3); per-basename audit executed and homed in `docs/plans/dsl-v2-defork-audit.md` §2A; activation gate set to the five-basename grounding de-fork (§4). Roster: deliberately empty (§3).
- Carried to the future resolver PR (not this model): confirm both import-binding assembly sites (~11761, ~12355) are the only type-binding merge loci (the `std.types`-special-cased branch folds key-by-key already and is naturally covered by the same checked-merge helper).
- Dependency owned elsewhere: the five-basename grounding de-fork (`algebra` authority ruled; `nat`/`integer`/`float` via #5428 numeric tower; `effects` axis decision) must complete before the guard PR can land green. `{node, coercion}` basename de-fork stays on the Route-C / v1-delete surface (audit category-(c)), unaffected by this guard.

## Dissolution trigger (DESIGN §6)

Delete this model-only DESIGN doc once the resolver type-name-collision wall is implemented and merged — the flag-ANY guard fires fail-closed on any same-name cross-authority type pair (TypeNameCollision diagnostic), the floor closure lands already-green after the de-fork, and the discriminating RED witness (synthetic divergent + byte-identical-mirror pairs both fire) is proven by execution — so this model is realized in the resolver.
