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

## Q4 — What needs to land (revised per Q5.5)

The closed-system foundation exists at the substrate level — but with substantive coverage gaps the audit must close:

- ✅ `Behavior` enum with the 5 primitives in `substrate.dag`. Live; complete.
- ⚠️ **Service primitives in `dsl/std/` + `dsl/extdeps/`: signature-shape coverage is INCOMPLETE today.** Some HTTP-derived primitives carry implicit effect signatures (via `derive_op_effect`'s method-table); many other effectful primitives (logging that returns Unit; mutation primitives that don't thread their target resource; etc.) do NOT carry the signature shape that would structurally express read-vs-write. Achieving full coverage is **required audit + substrate-work** under req 2 + req 3 below — NOT a current substrate fact. See Q5.5 for the deeper framing.
- ⚠️ `OperationEffect` taxonomy in `effects.dag:262-506` (Read/Upsert/Create/Append/Delete) — **status pending Q5.5 audit-as-existence-check** (see below). Either retained as a normalized view derived from signature shape, or retired as parallel-representation.
- ⚠️ `derive_op_effect` at `effects.dag:722-755` — same status; today derives from HTTP method + path. HTTP method IS a structural fact; whether the rest of the primitive surface aligns is the audit's existence-check.

**Honest live-state:** the closed-system FOUNDATION (5 behaviors + DAG substrate + the principle that operations should carry signature-shape) is live. The IMPLEMENTATION COVERAGE across all effectful primitives is partial. Req 2 + req 3 are the work that closes the gap.

What's needed:

1. **Effects lens** at `src/v3/lenses/effect_enumeration.dag` (or sibling), parallel to `cost.dag` precedent. Walks the 5-behavior structure; composes effect-classification per the Q2 table; surfaces as a structural fact on each function declaration. **Anchors on operation type-signature shape, not on hand-declared tags.**
2. **Audit-as-existence-check** (formerly "tag every primitive"): verify that every effectful primitive in `dsl/std/` + `dsl/extdeps/` has a type signature that structurally derives the right effect classification (returned-modified-resource indicates write; returns-derived-value-only indicates read; etc.). **If any primitive requires a hand-declared tag because its signature doesn't structurally reveal the effect, that's the existence-proof that taxonomy retirement is needed (Q5.5 path (ii))**. If all primitives derive cleanly from signature shape, taxonomy can be retained as a normalized view (Q5.5 path (i)).
3. **Resource-threading discipline for std/ primitives**: every external mutable resource must be modeled as a typed parameter that's returned modified (`log.info(msg, log: LogFile) → LogFile'`; sockets thread `Connection → Connection'`; etc.). Same pattern as IO-monad-style World-threading in pure-functional languages, without the monad — typed resource as parameter + returned-modified is the discipline. Forces the structural-signature-shape encoding at the substrate level. Audit (req 2) catches existing primitives that violate the discipline.
4. **Redundancy lens** (per aggressive reading) — referential-transparency proof for repeated identical reads with no intervening write. Compile-error-by-construction.
5. **`reread(key)` primitive** in std/ — structurally tags legitimate re-read for the optimistic-lock / refresh-on-stale cases the redundancy lens would otherwise reject.
6. **Transactional-pattern lens** — derived structural fact from Bind composition + typed transaction primitives (`Transaction → Transaction'`). Lens walks the Bind chain and recognizes the begin-modify-commit shape structurally. Same closed-system fold.
7. **No parser surface, no `effects [...]` clause, no annotation, no per-primitive tag declaration.** Effects are the structural fact (operation type-signature shape).

## Q4.5 — Pre-conditions (load-bearing for the closed-system claim)

The closed-system claim *"effects derive structurally from the composition of typed primitive operations"* is **honest only when typed primitives ARE the path**. Today, two structural holes exist where bypasses bypass the typed-primitive substrate. Both are pre-conditions for the lens to deliver on its closed-system claim — without them, the lens has structural holes wherever the foundation isn't yet closed.

### P1 — Extdeps typed-primitive consumption is structurally enforced

The substrate must make it impossible for a service definition to declare `messages: Json` instead of `messages: List<LlmMessage>`. Typed protocol carriers must be the ONLY path; raw Json / string-path extraction must have no structural home in service definitions.

**Today's tracked debt** (per ROADMAP.md:153-154):
- `dsl/extdeps/llm/openai.dag:92-110` — `messages: Json` + string-path output extraction (bypass: typed `LlmMessage` + `ContentBlock` carriers exist but not consumed).
- `dsl/extdeps/llm/anthropic.dag:104-124` — same pattern.
- `dsl/extdeps/github/auth.dag:13-24` — `github_token() → { token: Secret }` discards modeled scopes + `expires_at` from `GitHubAuthToken`.

**Why load-bearing**: until typed-primitive consumption is structurally enforced (impossible to bypass), service-definition surfaces can still declare effects via raw Json paths. The lens walking those declarations would have nothing structural to read; the closed-system claim has a hole exactly there. Required prereq under reqs 2 + 3 (audit catches; substrate-discipline closes).

**Sequencing**: prereq for the lens's *coverage* claim. The lens itself can land first (Q6 implementation brief); it just won't cover the bypassed extdeps surfaces until P1 closes. The lens's acceptance must surface structural-coverage-gap diagnostics on those surfaces (i.e., the lens itself reports "this surface has a structural hole; effect-derivation can't proceed structurally"). That's how the gap becomes load-bearing visible rather than silent.

### P2 — `ExecuteCommand` fully materialized as a typed runner primitive

Subprocess invocations need a typed primitive that derives "external-toolchain effect" structurally — analog of how `derive_op_effect` derives effects from HTTP method. Today the M1.5 testgen harness allowlists only `command == "true" && args.is_empty() && expect_exit == 0` and panics fail-closed on anything else; the Rust `TestRunner` returns `NotYetImplemented` for `ExecuteCommand` (see `TESTING.md:195` capability state callout).

**Why load-bearing**: TESTING.md (post-#782 cascade) committed to `0-residual` framing — boundary tests migrate to `ExecuteCommand`-based `.dag` `TestClaim` declarations. **Until `ExecuteCommand` can execute equivalent `.dag` TestClaims, deleting Rust boundary tests creates a verification gap.** Same closed-system shape applied to test execution: the typed runner primitive IS the path; subprocess invocations have nowhere structural to live until then.

**Sequencing**: pre-requisite for *any* Rust boundary-test deletion. Already named under PB-Runtime in Zero-Floor Manager's program scope; Director-signaled twice. Should be priority once Zero-Floor's authoring queue clears.

### Why these belong in this design doc

Both prereqs are load-bearing for the closed-system claim this doc commits to. Naming them in-doc surfaces the dependencies upfront rather than letting the implementation brief discover them as STOP-AND-ESCALATEs at dispatch time. Workers reading this doc see correctly that:
- The lens at Q6 can land independently (worth the empirical receipt of structural-coverage diagnostics).
- The closed-system claim's *full coverage* requires P1 + P2 closure.
- Without P1: bypass surfaces visible only via the lens's own gap diagnostics (lens lands; reports holes).
- Without P2: TESTING.md `0-residual` claim cannot be acted on (no Rust boundary-test deletion).

Worker-discretion-vs-Director-call:
- Lens implementation worker: dispatchable now; lens reports gaps as findings.
- P1 closure: substantive substrate work touching extdeps; tracked debt at ROADMAP.md:153-154; should land via dedicated lane.
- P2 closure: PB-Runtime in Zero-Floor; signal pending.

## Q5 — Asymmetric tightening (the only declaration-value exception)

The closed-system framing does NOT preclude callers from constraining what callees they're willing to invoke. Two exception cases retain declaration value:

- **Asymmetric tightening at caller**: a caller declares "I only invoke functions whose composed effects ⊆ {read-shaped}". Structural type matching at the call site fails for any callee whose body composes write-shaped operations. **Caller-side constraint, structural enforcement via type-signature shape, no separate lens.**
- **Override for test/correctness pinning**: rare; allows pinning a tighter contract for testing purposes. Same shape — caller-side constraint.

Neither is "annotate the callee with `effects [...]`" or "tag the callee primitive with OperationEffect". Both are caller-pins-tighter-than-derive.

## Q5.5 — OperationEffect taxonomy: retain as normalized view, or retire as parallel-representation?

**Status: OPEN; resolution pending audit (Q4 req 2).**

User's framing 2026-04-25 went one level deeper than the closed-system framing in Q1-Q5: *"having a Read effect might be a dual representation within our own language — i.e. the language should clearly know if it's reading something or writing something already"*.

The deeper observation: a function's type-signature shape already structurally carries whether it reads or writes:
- `service.get(id, svc) → User` (svc unchanged) is structurally a read.
- `service.insert(data, svc) → UserService'` (svc modified-and-returned) is structurally a write.

**The shape IS the effect.** A taxonomy `Read | Upsert | Create | Append | Delete` that names what the type signature already says is potentially `feedback_naming_is_aliasing` + `feedback_dissolve_bridges` + `feedback_parallel_representation_debt` firing at once.

### Two paths

**(i) OperationEffect taxonomy is a normalized view derived from signature shape.** Tags computed from operation type signatures (returned-modified-resource → write-shaped → tag as Upsert/Create; returns-derived-value-only → read-shaped → tag as Read). Per `feedback_naming_is_aliasing` — named types are namespaces over structural facts; tags are namespaces over the same. **Acceptable retention.** Idempotency lens + cost lens consumers continue to read tags; tags are computed from structure.

**(ii) OperationEffect tags are required-to-be-declared per primitive.** Then they're parallel-representation to what the signature already carries. **Retire entirely**; consumers walk type signatures directly. Bigger reframe — touches existing landed substrate (`effects.dag`'s enum + `derive_op_effect` + `idempotency.dag`'s anchor consume OperationEffect today). Stronger discipline-claim.

### Audit-as-existence-check (Q4 req 2 reframed)

The audit decides:
- **All effectful primitives in `dsl/std/` + `dsl/extdeps/` have type signatures that structurally derive the right effect classification** → path (i). Retain as normalized view; ensure no primitive declares its tag rather than deriving it.
- **At least one effectful primitive requires a hand-declared tag because its signature doesn't structurally reveal the effect** → path (ii). The existence proof shows the taxonomy is parallel-representation; retire it.

### Two design questions answered structurally (no new substrate)

PM surfaced two cases that might not map cleanly to type-signature shape:

1. **Operations whose external effects don't appear in their return type** — e.g., `log.info(msg)` returning Unit hides the file-write. **Resolution: every external mutable resource is modeled as a typed parameter that's returned modified.** `log.info(msg, log: LogFile) → LogFile'`. Sockets take `Connection`, return `Connection'`. File handles same. Forces the discipline at the type-signature level. Same pattern as Haskell IO monad's `World → (Result, World')` without the monad — typed resource threading is the discipline. **Existing primitives that violate this (e.g., logging that returns Unit) are the audit's existence-proof for path (ii).**

2. **Atomicity / transactional grouping** — not a single-op property; it's a property of the COMPOSITION. **Resolution: transactional pattern is a derived structural fact from Bind composition + typed transaction primitives (`Transaction → Transaction'`).** A lens walks the Bind chain and recognizes the begin-modify-commit shape. Same closed-system fold pattern.

Both answer structurally. No new substrate needed for either.

### Director recommendation

**Default expectation: path (ii).** User's deeper framing strongly suggests OperationEffect IS parallel-representation; the existence of `derive_op_effect` (deriving from HTTP method) is itself prior art that the structural fact is the authority and the tag is a renaming. Logging primitives that today return Unit are likely the audit's existence-proof.

**If the audit produces a counterexample where path (i) is genuinely cleaner**, surface it in the implementation PR for re-decision. The default is retire.

### THESIS amendment under (ii)

Stronger framing than the Q1 amendment: *"tracking effects as a separate taxonomy IS the bug pattern, dissolved by construction. Operations are intrinsically read-shaped or write-shaped or transactional via their type-signature shape; consumers walk the signatures directly; there is no parallel taxonomy to declare or maintain."*

## Q6 — Director-actionable recommendation

**Outcome: (a) closed-system framing.**

### Implementation-brief shape (replaces the retracted briefs)

Title: `feat(v3): T-ImpossibleBugs effects lens — closed-system structural derivation (compositional fold over 5 behaviors)`

**Reqs** (aligned with Q4's audit-as-existence-check + Q5.5's path (ii) default):

1. **Effects lens** at `src/v3/lenses/effect_enumeration.dag`, parallel to `cost.dag`. Walks the 5-behavior structure (Value / Transform / Branch / Loop / Bind); **anchors on operation type-signature shape**, not on hand-declared OperationEffect tags; composes effect classification per the Q2 table; surfaces structural-fact output (no annotation comparison, no parallel-taxonomy lookup).
2. **Audit-as-existence-check** — verify that every effectful primitive in `dsl/std/` + `dsl/extdeps/` has a type signature that structurally derives the right effect classification (returned-modified-resource → write-shaped; returns-derived-value-only → read-shaped). **NOT "tag every primitive"**. Per Q5.5: any primitive requiring a hand-declared `OperationEffect` tag because its signature doesn't structurally reveal the effect IS the existence-proof for path (ii) — taxonomy retirement.
3. **Resource-threading discipline applied to existing primitives** — primitives that violate the discipline today (e.g., logging that returns Unit instead of `LogFile → LogFile'`) get reshaped per the audit's findings. Foundation step toward signature-shape coverage.
4. **Redundancy lens** — referential-transparency proof for repeated identical reads with no intervening write. Emits compile-time `RedundantReadError` (Tier 1, not Tier 3).
5. **`reread(key)` primitive** in std/ for legitimate re-read cases. Structurally tagged. Single authority for re-read intent (per claude review observation: extend `reread` rather than adding parallel primitives if cache-invalidation / transactional-refresh ever need similar affordance).
6. **Transactional-pattern lens** — derived structural fact from Bind composition + typed transaction primitives (`Transaction → Transaction'`).
7. **Asymmetric-tightening worked example** in implementation PR body — concrete demonstration that caller-side constraint via structural type matching rejects a callee whose body composes write-shaped operations beyond the caller's pinned set. (Per claude review observation; the one place declaration-shaped surface re-enters deserves a worked example since "structural type matching at the call site" is doing nontrivial work.)
8. **Smoke + integration tests**: function with multiple typed-effect operations produces correct effect-set structural fact (derived from signature shape, not from tag lookup); redundant-read function fails compile; `reread`-using function compiles.

**STOPs**:
- **OperationEffect retirement decision (path (i) vs (ii))** — if audit (req 2) finds the existence-proof for path (ii) (any primitive whose signature doesn't structurally reveal its effect), STOP and surface the retirement scope to Director. Substrate retirement (`OperationEffect` enum + `derive_op_effect` + `idempotency.dag` re-anchor) is its own dedicated sub-lane; this lane should not absorb it. If audit confirms path (i) (all primitives derive cleanly from signature shape), surface that finding for re-decision.
- **Redundancy proof needs a `pure: Bool` carrier** — if the lens can't distinguish pure from impure Transforms inline, STOP. May need a sibling carrier on Transform targets. (Note: the deeper closed-system framing is that "pure" should also be derivable from signature shape — pure functions don't return modified resources — so this STOP may itself dissolve under further design.)
- **Asymmetric-tightening structural gap** — if caller can't actually pin effect-set constraints structurally today (i.e., the type system doesn't yet express "I require callee body's signature-shape composition ⊆ {read-shaped}"), STOP — that's its own substrate sub-lane.
- **Q4.5 P1 (extdeps typed-primitive bypass) — surfaced via lens findings**: lens reports structural-coverage-gap on `dsl/extdeps/llm/openai.dag:92-110`, `anthropic.dag:104-124`, `github/auth.dag:13-24` (and any others the audit finds). NOT a STOP; this is the lens delivering its closed-system-foundation-gap-visibility value. Director routes P1 closure to a dedicated extdeps-typed-primitive-consumption lane.
- **Q4.5 P2 (ExecuteCommand)** — the lens itself doesn't depend on P2; only the TESTING.md `0-residual` claim does. NOT a STOP for this lane. P2 is independent.

**Acceptance**: 4 worked examples from this design doc compile (or fail-compile) per their stated outcomes; lens output observable via standard lens-output infrastructure; lens reports structural-coverage-gap diagnostics on Q4.5 P1 bypass surfaces (gap becomes visible, not silent); audit produces existence-proof verdict (path (i) vs path (ii)) for Director re-decision on OperationEffect retirement; asymmetric-tightening worked example in PR body; cargo + clippy + fmt clean; DB-8 fixed-point converges.

## Capacity / sequencing impact

| Brief | Status |
|---|---|
| `t-impossiblebugs-unenumerated-effects-worker.md` (substrate-effects lens; was lens-vs-declaration) | **SUPERSEDED** — banner pointing here |
| `t-impossiblebugs-unenumerated-effects-parser-worker.md` (parser declared_effects clause) | **SUPERSEDED** — banner pointing here; no parser surface needed |
| `t-impossiblebugs-unenumerated-effects-fn-arrow-refactor-worker.md` (#805 brief; Fn→Arrow refactor) | **KEEP** — independent value (cleans `params + return_type` vestige); not effects-specific |
| Bulk std/ annotation lane (was Lane D) | **DISSOLVES** — no annotations to add |

Net: 1 chain of 3 implementation lanes collapses to:
- 1 implementation lane (effects lens) — anchors on signature shape, not on tag lookup.
- 1 audit-as-existence-check lane (verify primitives' signature-shape coverage; NOT "tag every primitive") — produces the path (i) vs (ii) verdict on `OperationEffect` retention.
- 1 sub-lane (`reread` primitive in std/ if not already there) + 1 transactional-pattern lens.
- 2 prereq lanes named in Q4.5: P1 extdeps typed-primitive consumption; P2 `ExecuteCommand` materialization. Both pre-existing tracked-debt; surfaced explicitly so the lens's closed-system claim is honest end-to-end.

Smaller scope; stronger claim. The taxonomy-retirement scope (substrate-side) is **not** in this lane — it's surfaced by audit and routed to dedicated retirement lane if path (ii) wins.

## Cross-manager note

- **Zero-Floor Manager**: heads-up. Effect-signature tagging on std/ primitives may touch substrate-adjacent territory; coordinate at audit-phase if needed.
- **Grounding Manager**: no current overlap.
- **PM**: synergy framing (5-behavior fold table) is the load-bearing reframe; the effects lens is positioned as the next instance of an established pattern (idempotency / termination / omni-emit are prior art) rather than novel substrate work.

## Closing signal

Director cleared to author the implementation brief based on Q6's shape. Worker dispatch follows. The retracted briefs get SUPERSEDED banners pointing here.
