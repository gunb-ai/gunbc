# T-Substrate sub-lane — Cardinality-for-int-lit subset `(M; substrate sub-lane scoping brief)`

> **Substrate Manager program brief.** Scopes the cardinality-substrate
> work needed to unblock T-Modeling int-lit magnitude (Goal 2). Per
> [`docs/r2-structure.md`](../r2-structure.md) Substrate Manager
> ownership; not full substrate-capability — narrowed to the int-lit
> consumer.
>
> **Producer:** Substrate Manager (this brief).
> **Consumer:** Modeling Manager — `r2-modeling-int-lit-magnitude-worker.md`
> (Wave 3).

## Read first

- **[`ROADMAP.md` §"Surface int-literals are concept-layer host-narrowed, not reconciliation-narrowed"](../../ROADMAP.md)** (`:355`) — the canonical statement of the gap. `IntLit(Int)` is already host-narrowed to `i64` at the tokenizer; magnitude bounds + cardinality refinement substrate is needed for unbounded literals.
- **[`docs/db-history/db-11.md`](../db-history/db-11.md)** — alias-`where` refinement substrate; current state of cardinality refinement. T-Modeling int-lit's expected carrier may share substrate with DB-11.
- **[PR #806](https://github.com/gunb-ai/gunbc/pull/806)** — "t-substrate cardinality int" (royal-badger-316, merged) — confirm the merged scope and what's residual; this brief authors only what #806 didn't cover.
- **[`dsl/std/types.dag`](../../dsl/std/types.dag)** — `kernel_primitives` (`:65-74`), `container_arity` (`:79-91`); `Int = Int64 = OrderedRing<Word64>` definition.
- **[`src/v3/std/tokenize.dag:30`](../../src/v3/std/tokenize.dag)** — `IntLit(Int)` declaration (host-narrowed).
- **[`src/v3/std/substrate.dag`](../../src/v3/std/substrate.dag)** — live substrate authority.
- **`feedback_audit_adjacent_authority_first`** — cardinality refinement substrate may already partially exist via DB-11 work; grep before designing.

## Frame

T-Modeling int-lit magnitude needs to express "this Int literal has magnitude N, bounded by [A,B]" structurally — so reconciliation can narrow at the type-algebra boundary instead of at the tokenizer. Today the tokenizer narrows to host `i64` immediately; `i64::MIN` as a single token is unrepresentable; magnitudes outside `i64` range have no concept-layer representation.

**The cardinality-for-int-lit substrate subset is the producer:** structural carrier(s) for int-literal magnitude bounds + cardinality refinement that T-Modeling int-lit consumes during narrowing.

**Scope is narrow:** only what T-Modeling int-lit needs. Full cardinality-substrate (e.g., for fixed-width types in `bit.dag`, for arbitrary-width arithmetic, for general refinement-by-cardinality) is **out of scope** for this sub-lane — those are separate substrate-capability lanes that may share design but don't share dispatch.

## Pre-author authority audit (mandatory)

**Before designing**, grep `src/v3/std/` + `src/v3/spec/` + #806 / #803 / #796 (closed) merge content for:

- existing magnitude / bound / cardinality-refinement carriers
- DB-11 alias-`where` parsing + lowering: how far did it land?
- `IntLiteralMagnitude` (rejected design from #796) — make sure the existing-authority audit doesn't mistakenly resurface it
- existing alias-form refinements (`type Int8 = Int where ...`) and how they lower today

**If audit reveals existing authority sufficient for T-Modeling int-lit's needs, reframe as consumer migration** — and the Substrate side of this lane may be effectively done.

## Open design questions (worker / Substrate Manager resolves at dispatch)

1. **Magnitude representation.** ROADMAP P4 debt row hints at "unbounded magnitude at concept layer" — possible carriers: `std.natural` magnitude-unbounded-natural type, or alias-form refinement on `Int` with `where` predicates. Worker picks; surface in PR.
2. **Reconciliation narrowing point.** Does narrowing happen at the type-checking boundary (during reconciliation), at the algebra-attachment point, or at tokenizer→parser handoff? Today it's at the tokenizer; the brief moves it later in the pipeline.
3. **`i64::MIN` representability.** ROADMAP cites the principled workaround `(0 - MAX - 1)` under `OrderedRing` additive-inverse. Does this lane formalize that as the canonical path, or does it land a true unbounded magnitude carrier? Both shapes acceptable; surface choice.
4. **Negation.** ROADMAP cites that no unary-minus parse primary is needed (`-x = 0 - x` desugars principally). Confirm this stays true under the new substrate; surface if not.

## Slice (worker fills at dispatch)

The structural design is mostly worker / Substrate Manager work; this brief sets boundaries:

1. **Audit existing cardinality / refinement substrate** (per audit section above).
2. **Land minimal substrate addition** in `src/v3/std/` to express int-literal magnitude bounds structurally.
3. **Lower the carrier** — tokenizer-side or reconciliation-side; surface choice.
4. **Coproduct dissolution receipt** for any new variant.
5. **No consumer migration in this PR** — that's T-Modeling int-lit's job (Wave 3 worker brief).
6. **Cross-program signal:** on merge, signal Modeling Manager via `Cross-manager notifications queued` that T-Modeling int-lit is dispatchable.

## Acceptance

- [ ] Authority audit receipt recorded; if existing authority sufficient, brief reframes as no-op or thin consumer-migration.
- [ ] Substrate carrier(s) for int-lit magnitude land in `src/v3/std/` (canonical authority + Rust mirror).
- [ ] Coproduct dissolution receipt for any new variant.
- [ ] Reconciliation narrowing point documented in PR body.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] Cross-program readiness signal posted to Modeling Manager.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.

## STOP-AND-ESCALATE

Surface to Substrate Manager / Director:

- **Scope expansion:** if the structural-design surfaces work that benefits more than just T-Modeling int-lit (e.g., a fully general cardinality-substrate that also unblocks fixed-width types or DB-11 alias-`where`) — that's a re-scope decision, not a quiet expansion. Surface for design call.
- **Audit reveals #796's `IntLiteralMagnitude` shape was substantively right, just dispatched-wrong.** If the audit favors revisiting the rejected design, surface explicitly with discipline-justification — the prior rejection was on parallel-representation grounds; if the audit shows it's NOT parallel-representation, the rejection may itself need revision.
- **DB-11 alias-`where` doesn't yet land cardinality refinement.** If the prerequisite is missing, surface as cross-substrate dependency; T-Modeling int-lit may need to wait on DB-11 closure.

## Non-goals

- **Not full cardinality-substrate.** Fixed-width types (`bit.dag`), arbitrary-width arithmetic, general refinement-by-cardinality — separate lanes.
- Not alias-`where` parsing/lowering completion (DB-11 territory; coordinate, don't subsume).
- Not reconciliation-pipeline redesign — minimal targeted change to admit the magnitude carrier.

## Cross-program note

- **Producer:** Substrate Manager (this brief).
- **Consumer:** Modeling Manager → `r2-modeling-int-lit-magnitude-worker.md` (gated on this lane's readiness signal).
- **Adjacent:** DB-11 alias-`where` / fixed-width types (P4 debt row at ROADMAP `:354`); coordinate but don't subsume.

## Reporting

Single PR. Title: `feat(v3): T-Substrate cardinality-for-int-lit subset — minimal magnitude carrier for T-Modeling int-lit consumer`. Body cites this brief + audit receipt + reconciliation narrowing decision + cross-program signal.

On merge: signal Modeling Manager that int-lit consumer is dispatchable.
