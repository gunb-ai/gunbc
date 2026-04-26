# T-Modeling — Int-literal magnitude worker brief `(M; consumer of t-substrate-cardinality-int-lit-worker.md range-facts-only scope)`

> **Worker brief.** Reports through Modeling Manager (post-R2 spin-up) /
> Director (pre-spin-up). T-Modeling Goal 2 item per
> [`docs/r2-structure.md`](../r2-structure.md).
>
> **Gated on:** Substrate Manager readiness signal for
> [`t-substrate-cardinality-int-lit-worker.md`](t-substrate-cardinality-int-lit-worker.md).
> **Do not dispatch until that signal posts.**
>
> **Scope reconciled with producer (post-`wise-pike-578` re-scope):**
> producer KEEPS `LiteralBits::Int(i64)` substrate-side; ranges are
> String-decimal facts; `i64::MIN`-as-single-token deferred to a
> sibling Int128/Word128 sub-lane. **This consumer brief operates
> against the i64-bounded carrier per producer's re-scope** — it does
> NOT widen IntLit, does NOT close `i64::MIN` representability, and
> does NOT ship anything the producer's deferred sibling sub-lane has
> not yet landed.

## Read first

- **[`docs/briefs/t-substrate-cardinality-int-lit-worker.md`](t-substrate-cardinality-int-lit-worker.md)** — **canonical producer brief** (post-`wise-pike-578` re-scoped). Read in full; this consumer brief consumes ONLY what producer landed: range-facts (String-decimal) + reconciliation narrowing + `MagnitudeOutOfRange` diagnostic, all against existing `LiteralBits::Int(i64)`.
- **[`docs/briefs/r2-substrate-cardinality-for-int-lit-subset.md`](r2-substrate-cardinality-for-int-lit-subset.md)** — closed-as-redundant routing doc; the producer authority is the `t-substrate-cardinality-int-lit-worker.md` brief above.
- **[`ROADMAP.md` §"Surface int-literals are concept-layer host-narrowed"](../../ROADMAP.md)** (`:355`) — the gap statement; the i64-bounded scope of this lane is the partial closure.
- **[`src/v3/std/tokenize.dag:30`](../../src/v3/std/tokenize.dag)** — `IntLit(Int)` declaration; **stays as-is** per producer's re-scope.
- **[PR #806](https://github.com/gunb-ai/gunbc/pull/806)** — t-substrate cardinality int (merged); confirm what landed and what's still consumer-side work.
- **[`THESIS.md`](../../THESIS.md)** — int-literal magnitude as Tier 1 thesis claim (note: full Tier 1 closure including `i64::MIN`-as-single-token requires the sibling Int128/Word128 sub-lane; this lane partial-closes).

## Frame

Producer landed: range facts (String-decimal `range_min_inclusive` / `range_max_inclusive` per integer algebra) + reconciliation magnitude-aware narrowing at `infer.rs:704-725` + `MagnitudeOutOfRange` typed diagnostic, **all against the existing `LiteralBits::Int(i64)` carrier**. Carrier widening (Int128/UInt128/Word128Carrier) is deferred to a sibling sub-lane.

This consumer brief migrates downstream consumers (call sites that emit, query, or pattern-match on int-literal magnitude) to consume the new substrate facts where appropriate. **It does NOT widen `LiteralBits::Int`**; the i64-bounded carrier is producer's authority.

## Slice (post-re-scope; range-facts consumption only)

1. **Confirm Substrate readiness signal** from producer brief's PR. Read producer's PR body to confirm: range-facts shape (String-decimal), narrowing site (infer.rs:704-725), `MagnitudeOutOfRange` diagnostic placement.
2. **Audit downstream consumers** of int-literal magnitude — emitters, lens evaluators, codegen sites that do magnitude-based dispatch. Output: one-row-per-consumer table noting whether it consumes the new range-facts via the producer's narrowing or whether it operates downstream of narrowing without further consumption.
3. **Migrate consumers** that benefit from the new facts:
   - Per-target emitter realizations that pick narrower target types based on literal magnitude (e.g., `let x: u8 = 5` should now reach Word8 idiomatically post-narrowing).
   - Any lens consumer that previously assumed `Int64` for all int literals.
4. **Test coverage** — add or update tests proving the narrowing reaches consumers correctly:
   - `let x: u8 = 5` produces u8-typed code at emit, not Int64.
   - `let x: u8 = 256` produces `MagnitudeOutOfRange` diagnostic (already in producer; this brief verifies consumer-side reporting works).
5. **DB-8 fixed-point bit-identical** for all existing programs (existing programs that didn't trigger narrowing should be unaffected; programs that DID benefit from narrowing now produce narrower target code — surface any in PR body).

## Acceptance

- [ ] Downstream-consumer audit recorded in PR body (table per Slice §2).
- [ ] Consumer migrations land per Slice §3 audit.
- [ ] Test coverage for narrowing reaching consumers correctly.
- [ ] DB-8 fixed-point converges bit-identically (or surfaces any intentional drift from narrowing benefits).
- [ ] Cross-program signal back: lane close → Modeling Manager → R2 Release Manager. **Note:** Goal 2 int-lit closure is **partial** (i64-bounded); full closure including `i64::MIN`-as-single-token requires the sibling Int128/Word128 sub-lane. Surface this in cross-program signal so R2 Release Manager understands the scoped close.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.
- [ ] No `--no-verify` push without explicit cargo-unavailable note.

## STOP-AND-ESCALATE

- **Producer brief landed a different carrier shape than this brief assumes** (e.g., re-scope reverted, or new fields added to range-facts). Re-read producer; this brief's slice may need adjustment.
- **Audit reveals a consumer that genuinely needs the deferred Int128/Word128 carrier** (i.e., its dispatch logic requires representing magnitudes outside i64). STOP — that consumer waits on the sibling sub-lane; do not partial-migrate it.
- **Reconciliation narrowing site changed** between producer's authoring and worker's dispatch. Surface; producer brief may need refresh.
- **DB-8 drifts unexpectedly** — STOP immediately.
- **`i64::MIN`-as-single-token tests requested** — that's the deferred sibling sub-lane's scope; do NOT add such tests in this PR.

## Non-goals

- **NOT widening `LiteralBits::Int(i64)`** — explicitly producer's deferred sibling sub-lane.
- **NOT closing `i64::MIN`-as-single-token representability** — same.
- Not extending int-algebra semantics.
- Not authoring the producer carrier (producer brief's job).
- Not tokenizer-charclass phase-2 (separate T-Modeling item).
- Not full DB-11 alias-`where` closure (adjacent, not subsumed).

## Cross-program note

- **Producer:** Substrate Manager → `t-substrate-cardinality-int-lit-worker.md` (range-facts-only re-scope).
- **Consumer:** this brief (Modeling Manager).
- **Sibling sub-lane (deferred; not part of this lane):** Int128/UInt128/Word128Carrier substrate work that closes `LiteralBits::Int` widening + `i64::MIN`-as-single-token. Tracked separately by Substrate Manager.
- **Downstream signal:** R2 Release Manager — Goal 2 int-lit **partial close** (i64-bounded scope). Full close requires sibling Int128/Word128 sub-lane completion.

## Reporting

Single PR. Title: `feat(v3): T-Modeling int-lit magnitude — consume range-facts substrate; narrowing reaches downstream consumers (i64-bounded)`. Body cites this brief + producer brief + signal-receipt + downstream-consumer audit table + DB-8 disposition + explicit note that full Goal 2 closure remains gated on sibling Int128/Word128 sub-lane.

On merge: signal R2 Release Manager — Goal 2 partial close.
