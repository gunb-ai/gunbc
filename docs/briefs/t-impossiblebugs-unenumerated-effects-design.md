# T-ImpossibleBugs unenumerated effects — design/scoping doc (closed-system framing)

> **Output of Director scoping pass per user + PM 2026-04-25 exchange.**
> Replaces the lens-vs-declaration framing of
> [`t-impossiblebugs-unenumerated-effects-worker.md`](t-impossiblebugs-unenumerated-effects-worker.md)
> + [`t-impossiblebugs-unenumerated-effects-parser-worker.md`](t-impossiblebugs-unenumerated-effects-parser-worker.md)
> with a closed-system structural-derivation framing that aligns with how
> complexity / idempotency / termination already work in v3.
>
> Both prior briefs are now SUPERSEDED. No code has merged against them
> yet (only briefs landed; `t-impossiblebugs-unenumerated-effects-fn-arrow-refactor-worker.md`
> from #805 is independent value and stays dispatchable).

## TL;DR

**Recommendation: closed-system structural derivation, no annotation surface.**

Effects in gunbc are not annotations. They derive structurally from the composition of typed primitive operations, exactly the way complexity is derived from `fold` / `descend` / `repeat` composition. Same closed-system mechanism, different property. Effects lens is the next instance of an established pattern (idempotency, termination, omni-emit are prior art), not novel substrate work.

The bug class **closes more strongly** under closed-system framing: silent effect leakage is impossible because there's no separate "declared" surface to leak from — the structural fact IS the declaration.

## Q1 — The closed-system invariant

Per `feedback_compiler_is_dag_processor`: the compiler knows only `Node / Conj / Disj / Cardinality / Bit`. Per `substrate.dag`, all computation composes from 5 primitive behaviors:

`Value | Transform | Branch | Loop | Bind`

There is no `while(true)`, no unbounded recursion, no `eval(string)`, no raw IO escape hatch. Every form of computation MUST appear as one of these 5. The substrate is closed; the lens domain is closed; **nothing can hide**.

For effects specifically: every external interaction (db read/write, file IO, network call, log) goes through a typed service primitive declared in `dsl/std/` or `dsl/extdeps/`. Service declarations carry effect signatures (`derive_op_effect` at `effects.dag:722-755` already does this for HTTP). There is no raw effect — no way to perform a side effect without going through a typed primitive.

## Q2 — Every lens is a compositional fold over the 5 behaviors

This is the unifying mechanism. Same shape every time:

| Behavior | Complexity | Effects | Idempotency | Termination |
|---|---|---|---|---|
| **Value** | O(1) | ∅ | identity | trivial |
| **Transform** | callable cost (recursive walk) | callable effect signature (recursive walk) | callable idempotency | requires witness if recursive |
| **Branch** | max(arms) | union(arms) | conjunction(arms) | descend witness on loop arm |
| **Loop** | bound × body | body's effects | body must be idempotent for outer to be | descend witness mandatory |
| **Bind** | sum(bindings) + body | union(bindings) ∪ body | sequential composition rule | each binding's witness composes |

**Anything compositional + structural is a fold over these 5.** The closed-system invariant means every form of computation MUST appear as one of these 5; therefore the lens domain is closed; therefore nothing can hide.

## Q3 — Worked examples

### Example 1 — complexity (already implemented; cited as foundation)

```
fn merge_sort(xs: List<Int>) -> List<Int> =
  descend xs by ListShrink {
    case [] => []
    case [single] => [single]
    case _ =>
      let (left, right) = split_in_half(xs)
      merge(merge_sort(left), merge_sort(right))
  }
```

Complexity lens walks structurally: `descend` → log₂(N) depth (each step halves); each level: linear `split_in_half` + linear `merge` (sum-of-children); composes via T(N) = 2·T(N/2) + O(N) → **O(N log N)**.

User can declare `complexity merge_sort: O(N log N)` to *check* the lens-computed bound. **No annotation needed**; the lens computes regardless.

### Example 2 — effects (target shape under closed-system)

```
fn fetch_and_log(user_id: UserId) -> User =
  let user = user_service.get(user_id)        // typed primitive: ReadEffect<User>
  log.info("Fetched user {}", user.name)       // typed primitive: LoggingEffect
  user
```

Effects lens walks structurally:
- `Bind` over two bindings: `let user = ...` + `log.info(...)`
- First Transform target = `user_service.get` → `ReadEffect<User>`
- Second Transform target = `log.info` → `LoggingEffect`
- Composition (per Bind row in the table): union(bindings) ∪ body = `{ ReadEffect<User>, LoggingEffect }`

The lens surfaces this as a structural fact on the function declaration. **No `effects [Read, Logging]` clause needed**; the set is the structure.

### Example 3 — redundancy (referential-transparency proof)

```
fn upsert_user_buggy(id: UserId, data: UserData) -> Result<()> =
  let existing1 = user_service.get(id)       // Bind { Transform: ServiceGet(id) }
  let existing2 = user_service.get(id)       // Bind { Transform: ServiceGet(id) } — same target, same args
  validate(data)                              // Transform: validate — pure, ∅ effect
  user_service.insert(data)                   // Transform: ServiceInsert(data)
  transaction.commit()                        // Transform: TransactionClose
```

A redundancy lens walks the Bind sequence:
1. `existing2`'s Transform target = `ServiceGet`, args = `id`.
2. Walk back through Bind sequence to `existing1`'s Transform target = `ServiceGet`, args = `id`. **Same target, same args.**
3. Walk Transforms between them: `validate(data)` — Transform target's effect set = ∅ (pure). No write-effect intervened.
4. **Conclusion**: `existing2 ≡ existing1` by referential transparency. Structurally provable — not a heuristic.

### Aggressive vs conservative reading on Example 3

- **Conservative**: lens surfaces the redundancy as a finding; not a compile error. User can choose to address.
- **Aggressive (closed-system discipline-strongest)**: redundant read is a **compile error by construction**. To express LEGITIMATE re-read (optimistic-lock retry, refresh-on-stale), the user calls an explicit `reread(key)` primitive in std/ that structurally tags "I know this is a re-read."

**Director picks aggressive.** Aligns with THESIS Tier-1 impossible-by-construction commitment + `feedback_construction_over_ratchets` (model first; violations dissolve). The cost (`reread` primitive) is bounded; the discipline-strength is much higher. Conservative is softer than the discipline supports.

### Example 4 — idempotency (already implemented; prior art)

`idempotency.dag` already walks workflow effect composition + computes idempotency from primitive algebraic properties. Same fold pattern as the other lenses; cited as proof the pattern works.

## Q4 — What needs to land

This is **not** a substrate-extension lane. The substrate already exists:

- ✅ `OperationEffect` taxonomy in `effects.dag` (Read/Upsert/Create/Append/Delete).
- ✅ `derive_op_effect` at `effects.dag:722-755` already derives effects from HTTP method + path structurally (foundation of the closed-system shape).
- ✅ Service primitives in `dsl/std/` + `dsl/extdeps/` carry typed effect signatures.
- ✅ `Behavior` enum with the 5 primitives in `substrate.dag`.

What's needed:

1. **Effects lens** at `src/v3/lenses/effect_enumeration.dag` (or sibling), parallel to `cost.dag` precedent. Walks the 5-behavior structure; composes effect set per the Q2 table; surfaces as a structural fact on each function declaration.
2. **Generalize effect-derivation beyond HTTP** — any typed primitive that performs a side effect carries an effect signature (not just HTTP-derived ones). Audit existing `dsl/std/` primitives + tag.
3. **Redundancy lens** (per aggressive reading) — referential-transparency proof for repeated identical reads with no intervening write. Compile-error-by-construction.
4. **`reread(key)` primitive** in std/ — structurally tags legitimate re-read for the optimistic-lock / refresh-on-stale cases the redundancy lens would otherwise reject.
5. **No parser surface, no `effects [...]` clause, no annotation**. Effects are the structural fact.

## Q5 — Asymmetric tightening (the only declaration-value exception)

The closed-system framing does NOT preclude callers from constraining what callees they're willing to invoke. Two exception cases retain declaration value:

- **Asymmetric tightening at caller**: a caller declares "I only invoke functions whose effect set ⊆ {Read}". Structural type matching at the call site fails for any callee whose body composes effects beyond Read. **Caller-side constraint, structural enforcement, no separate lens.**
- **Override for test/correctness pinning**: rare; allows pinning a tighter contract for testing purposes. Same shape — caller-side constraint.

Neither is "annotate the callee with `effects [...]`". Both are caller-pins-tighter-than-derive.

## Q6 — Director-actionable recommendation

**Outcome: (a) closed-system framing.**

### Implementation-brief shape (replaces the retracted briefs)

Title: `feat(v3): T-ImpossibleBugs effects lens — closed-system structural derivation (compositional fold over 5 behaviors)`

**Reqs:**

1. **Effects lens** at `src/v3/lenses/effect_enumeration.dag`, parallel to `cost.dag`. Walks the 5-behavior structure (Value / Transform / Branch / Loop / Bind); composes effect set per the Q2 table; surfaces structural-fact output (no annotation comparison).
2. **Audit + tag std/ primitives** — every effectful primitive in `dsl/std/` carries an explicit `OperationEffect` signature. Generalizes `derive_op_effect`'s HTTP-derivation to all primitives.
3. **Redundancy lens** — referential-transparency proof for repeated identical reads with no intervening write. Emits compile-time `RedundantReadError` (Tier 1, not Tier 3).
4. **`reread(key)` primitive** in std/ for legitimate re-read cases. Structurally tagged.
5. **Smoke + integration tests**: function with multiple typed-effect calls produces correct effect-set structural fact; redundant-read function fails compile; `reread`-using function compiles.

**STOPs**:
- If audit (req 2) reveals a primitive performing side effects without an `OperationEffect` tag (a hidden hole in the closed-system invariant), STOP — that's a substrate gap that needs filling first.
- If the redundancy proof requires substrate primitives the lens can't read structurally (e.g., distinguishing pure from impure transforms inline), STOP — may need a `pure: Bool` fact or sibling carrier on Transform targets.
- If asymmetric-tightening at caller surfaces a structural-type-matching gap (caller can't actually pin effect-set constraints structurally today), STOP — that's its own substrate sub-lane.

**Acceptance**: 4 worked examples from this design doc compile (or fail-compile) per their stated outcomes; lens output observable via standard lens-output infrastructure; cargo + clippy + fmt clean; DB-8 fixed-point converges.

## Capacity / sequencing impact

| Brief | Status |
|---|---|
| `t-impossiblebugs-unenumerated-effects-worker.md` (substrate-effects lens; was lens-vs-declaration) | **SUPERSEDED** — banner pointing here |
| `t-impossiblebugs-unenumerated-effects-parser-worker.md` (parser declared_effects clause) | **SUPERSEDED** — banner pointing here; no parser surface needed |
| `t-impossiblebugs-unenumerated-effects-fn-arrow-refactor-worker.md` (#805 brief; Fn→Arrow refactor) | **KEEP** — independent value (cleans `params + return_type` vestige); not effects-specific |
| Bulk std/ annotation lane (was Lane D) | **DISSOLVES** — no annotations to add |

Net: 1 chain of 3 implementation lanes collapses to 1 implementation lane (effects lens) + 1 sub-lane (`reread` primitive in std/ if not already there) + 1 audit lane (tag std/ primitives with effect signatures). Smaller scope; stronger claim.

## Cross-manager note

- **Zero-Floor Manager**: heads-up. Effect-signature tagging on std/ primitives may touch substrate-adjacent territory; coordinate at audit-phase if needed.
- **Grounding Manager**: no current overlap.
- **PM**: synergy framing (5-behavior fold table) is the load-bearing reframe; the effects lens is positioned as the next instance of an established pattern (idempotency / termination / omni-emit are prior art) rather than novel substrate work.

## Closing signal

Director cleared to author the implementation brief based on Q6's shape. Worker dispatch follows. The retracted briefs get SUPERSEDED banners pointing here.
