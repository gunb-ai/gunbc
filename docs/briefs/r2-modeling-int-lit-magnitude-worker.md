# T-Modeling — Int-literal magnitude worker brief `(M; consumer of T-Substrate cardinality-for-int-lit)`

> **Worker brief.** Reports through Modeling Manager (post-R2 spin-up) /
> Director (pre-spin-up). T-Modeling Goal 2 item per
> [`docs/r2-structure.md`](../r2-structure.md).
>
> **Gated on:** Substrate Manager readiness signal for
> [`t-substrate-cardinality-int-lit-worker.md`](t-substrate-cardinality-int-lit-worker.md).
> **Do not dispatch until that signal posts.**

## Read first

- **[`docs/briefs/r2-substrate-cardinality-for-int-lit-subset.md`](r2-substrate-cardinality-for-int-lit-subset.md)** — producer brief; defines the magnitude carrier this worker consumes.
- **[`ROADMAP.md` §"Surface int-literals are concept-layer host-narrowed"](../../ROADMAP.md)** (`:355`) — the gap statement.
- **[`src/v3/std/tokenize.dag:30`](../../src/v3/std/tokenize.dag)** — `IntLit(Int)` declaration; the host-narrowed point that this brief moves later in the pipeline.
- **[PR #806](https://github.com/gunb-ai/gunbc/pull/806)** — t-substrate cardinality int (merged); confirm what landed and what's still consumer-side work.
- **[`THESIS.md`](../../THESIS.md)** — int-literal magnitude as Tier 1 thesis claim.

## Frame

Producer (T-Substrate cardinality-for-int-lit) lands the magnitude carrier. This brief migrates consumers — tokenizer, parser, reconciliation, and any int-literal consumer — to consume the new substrate instead of the host-narrowed `i64`.

The narrowing point moves from tokenizer (today) to reconciliation (target). After this lane closes, `i64::MIN` and other unbounded literals are representable at concept layer; reconciliation narrows to the target int algebra at the type-checking boundary, not the lexer.

## Slice

1. **Confirm Substrate readiness signal.** Read the producer brief's PR; confirm magnitude carrier is in `src/v3/std/` with Rust mirror parity. If signal not yet posted, do not dispatch.
2. **Migrate `IntLit` representation.** Update `src/v3/std/tokenize.dag` (and Rust mirrors) to consume the magnitude carrier instead of host `i64`.
3. **Move the narrowing point.** Where today the tokenizer narrows to `i64`, reconciliation now narrows to the target int algebra. Update reconciliation pipeline accordingly.
4. **Diagnostic for out-of-range.** When a literal exceeds the target algebra's range (e.g., `2^64` against `Word64`), reconciliation emits a typed diagnostic (`MagnitudeOutOfRange` or worker-equivalent). Per `feedback_fail_closed_discipline` C-8.
5. **Regression tests:**
   - `i64::MIN` parses as a single literal token (concept layer admits it).
   - `2^64` against `Word64` produces typed diagnostic at reconciliation, not at lex.
   - Existing arithmetic on small literals stays bit-identical.
6. **DB-8 fixed-point bit-identical** for all existing programs.

## Acceptance

- [ ] `IntLit` representation migrated from host `i64` to magnitude carrier.
- [ ] Narrowing point moved from tokenizer to reconciliation.
- [ ] `MagnitudeOutOfRange` (or equivalent) typed diagnostic on out-of-range literals.
- [ ] Regression tests cover `i64::MIN` admittance + out-of-range diagnostic + existing-program bit-identity.
- [ ] DB-8 fixed-point converges bit-identically.
- [ ] Cross-program signal back: this lane's close gates Goal 2 item completion; signal Modeling Manager → R2 Release Manager.
- [ ] `cargo test --workspace --exclude v2-compiler-tests` / `cargo clippy --all-targets -- -D warnings` / `cargo fmt --all --check` clean.
- [ ] No `--no-verify` push without explicit cargo-unavailable note.

## STOP-AND-ESCALATE

- **Producer brief landed a different carrier shape than this brief assumes.** Re-read producer; this brief's slice may need adjustment.
- **Reconciliation narrowing point breaks more than expected** — surface; the change may need to land alongside other reconciliation work.
- **Existing programs drift on DB-8.** STOP immediately.
- **Negation desugar (`-x = 0 - x`)** breaks under the new substrate. Per ROADMAP, `OrderedRing` additive-inverse covers it; if not, surface — that's a substrate gap, not a consumer gap.

## Non-goals

- Not extending int-algebra semantics.
- Not authoring the magnitude carrier (producer brief's job).
- Not tokenizer-charclass phase-2 (separate T-Modeling item).
- Not full DB-11 alias-`where` closure (adjacent, not subsumed).

## Cross-program note

- **Producer:** Substrate Manager → cardinality-for-int-lit subset.
- **Consumer:** this brief (Modeling Manager).
- **Downstream signal:** R2 Release Manager — Goal 2 item close.

## Reporting

Single PR. Title: `feat(v3): T-Modeling int-lit magnitude — consume cardinality-for-int-lit substrate; narrow at reconciliation`. Body cites this brief + producer brief + signal-receipt cite + DB-8 disposition.

On merge: signal R2 Release Manager that Goal 2 int-lit item closed.
