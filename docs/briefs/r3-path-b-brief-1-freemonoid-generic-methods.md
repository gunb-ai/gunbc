---
status: PM-authored worker brief (deep-wolf-155 direct dispatch)
authority_parent: Operator briansrls 2026-05-15 directive — "do path B for tokenize/parse NO workarounds, spawn workers under you directly"
authoring_date: 2026-05-15
brief_set: docs/r3-path-b-tokenize-parse-brief-set.md (§Brief 1)
worker_session: witty-moth-725
reporting: directly to deep-wolf-155 via dashboard-message (no Director/Mgr intermediary)
---

# Path B Brief 1 — Substrate-Language: Generic Methods on `FreeMonoid<T>` (non-endomorphic map + per-method type params)

## Context

This brief is part of the Path B tokenize/parse retirement program (see `docs/r3-path-b-tokenize-parse-brief-set.md` for the full 9-brief set). The operator has chosen the NO WORKAROUNDS path — we're addressing the substrate-language gaps properly rather than working around them.

**Workers under deep-wolf-155 directly**: report findings + blockers via `dashboard-message send --to deep-wolf-155 --body "..."`. Do NOT route through Director/Mgr layer for this lane.

## The named gap

`dsl/std/algebra.dag:387-393` carries an explicit comment naming the substrate-language gap:

```
//   map: endomorphic fn(T) -> T -> FreeMonoid<T> (not fn(T)->U with U free)
//   fold: monoid-shaped on T — init: T plus step fn(T, T) -> T -> T
//   (Those two stay on T because executable authority here cannot yet name
//    per-method result/accumulator type parameters — same emitter gap as
//    lattice lifting above.)
```

`FreeMonoid<T>.map` is currently constrained to `fn(T) -> T` (endomorphic). What we need:
- `FreeMonoid<T>.map<U>(fn(T) -> U) -> FreeMonoid<U>` — non-endomorphic, per-method `U`.
- `FreeMonoid<T>.fold<Acc>(init: Acc, fn(Acc, T) -> Acc) -> Acc` — Acc-typed accumulator.

The comment explicitly names this as "executable authority here cannot yet name per-method result/accumulator type parameters." So the gap is at the EXECUTABLE substrate-language level — somewhere in the parse → lower → infer → emit pipeline.

## Why this matters for Path B

`regen_tokenize.rs` line ~503: `collect_keyword_rows(dag, shared_syntax: &SharedSyntaxAuthority) -> Vec<(String, String)>` — walks dag declarations + extracts keyword rows by mapping each `Declaration` to a `(name, kind)` tuple. That's `fn(Declaration) -> Tuple<String, String>` — non-endomorphic map. Without this substrate-language feature, the `.dag` codegen driver cannot express this walk cleanly.

Same pattern appears throughout regen_tokenize.rs / regen_parse_tables_emit.rs / regen_parse_emit.rs. Without non-endomorphic map, Briefs 7-9 (driver authoring) cannot proceed.

## Scope of this brief

### Phase A — Investigation (first deliverable)

Investigate WHERE in the pipeline the gap lives:

1. **Parser**: does `dsl/std/algebra.dag` syntax even accept `fn(T) -> U` in `FreeMonoid<T>.map`? Try authoring `map: fn<U>(fn(T) -> U) -> FreeMonoid<U>` (or whatever the syntax extension looks like) and see if it parses.
2. **Lower**: does lower process per-method type params? Look at how method declarations are lowered in `src/v3/compiler/src/lower.rs` (or — likely — in `lower_helpers_generated.rs` if substrate-driven).
3. **Infer**: does inference unify per-method type variables? Look at `src/v3/compiler/src/infer.rs` for type parameter binding.
4. **Emit**: does emit render generic-method monomorphizations? Look at `src/v3/compiler/src/emit/rust_target.rs` for trait/generic-method emission.

Surface findings to deep-wolf-155 via `dashboard-message send --to deep-wolf-155 --body "..."` BEFORE authoring fix. The investigation result determines the scope of the fix.

### Phase B — Land the fix

Once the investigation identifies the stage(s) needing changes:
1. Author substrate-language extension(s) at the identified stage(s).
2. Land per-method type parameter support so `FreeMonoid<T>.map<U>(fn(T) -> U) -> FreeMonoid<U>` works end-to-end.
3. Land analogous fix for `FreeMonoid<T>.fold<Acc>`.
4. Update `dsl/std/algebra.dag` removing the line-387-393 comment naming the gap.

### Phase C — Test fixture

Author a `.dag` fixture that demonstrates non-endomorphic map:

```dag
data sample_ints: List<Int> = [1, 2, 3]
data sample_strings: List<String> = sample_ints.map<String>(int_to_string)  // assuming int_to_string exists per Brief 2
```

The fixture should lower + infer + emit cleanly + execute correctly.

## Deliverables (concrete)

1. **Investigation report** to deep-wolf-155 via dashboard-message: which pipeline stage(s) own the gap.
2. **Substrate-language change PR** landing per-method type params on FreeMonoid methods + analogous parametric carriers (if any).
3. **Test fixture PR** demonstrating non-endomorphic map works end-to-end.
4. **dsl/std/algebra.dag comment removal**: lines 387-393 naming the gap are deleted in the same PR as the fix.

## Acceptance criteria (substrate-fact-at-HEAD)

- `cargo test -p v3-compiler --test integration generic_method_type_params_test` passes (test name TBD).
- A `.dag` fixture demonstrates `List<Int>.map<String>(int_to_string)` and the lens-fold compiles + executes correctly.
- `grep -n "executable authority here cannot yet name per-method" dsl/std/algebra.dag` returns 0 matches (the gap-naming comment is gone because the gap is closed).
- No new hand-Rust escorts added to satisfy this brief (Phase 1 anti-paper-shrink stance — substrate growth is the receipt, not Rust file growth).

## Anti-paper-shrink check

Naive workarounds that DO NOT count:
- Adding a separate `map_to<U>` method alongside `map` (creates parallel authority — fix the existing `map`)
- Authoring the fix only in Rust (`infer.rs` etc.) without `.dag` substrate change — substrate must grow alongside Rust
- Moving the limitation comment to a different file (the limitation must be GONE, not relocated)

The discriminator: this fix PASSES only if the existing `FreeMonoid<T>.map` declaration in `dsl/std/algebra.dag` is upgraded in place (or replaced with a more expressive declaration) AND the dependent pipeline stages flow that change through.

## Risks + open questions to surface back

If investigation reveals:
- The gap is at the **parser** level → likely 2-6 months substrate-language work (grammar extension)
- The gap is at the **lower/infer** level → may be tractable in weeks if it's a missing binding pass
- The gap is at the **emit** level → may require Rust target trait/generic emission work (longer)

Surface the SPECIFIC stage + investigation findings BEFORE deciding scope. If the fix is much larger than this brief assumes, re-scope with deep-wolf-155.

Other risks:
- Existing call sites that already assume endomorphic map may need migration after the fix lands. Audit + count migration sites during investigation.
- `fold<Acc>` may introduce an Acc≠T illegal-state class — verify whether the runtime stack can handle Acc-typed values that aren't T.

## Coordination

- **Report findings** to deep-wolf-155 via `dashboard-message send --to deep-wolf-155 --body "..."` after Phase A investigation.
- **Pause for guidance** before Phase B if investigation reveals scope substantially larger than this brief assumes.
- **Tag PRs** with title prefix `r3-path-b-brief-1: ...` for traceability.
- **Coordinate with sibling Briefs 2-3** (workers `sunny-tern-495` and `bright-swift-668`) if work overlaps — message peer workers directly if needed.

## Estimated effort

2-6 months substrate-language work. Investigation-first (1-2 weeks) clarifies which stage owns the gap.

## Read first

- `dsl/std/algebra.dag` lines 380-420 (FreeMonoid<T> declaration + the gap comment)
- `dsl/std/string_type.dag` (String = FreeMonoid<Char>)
- `docs/r3-path-b-tokenize-parse-brief-set.md` (full brief set context — operator's NO WORKAROUNDS framing)
- `src/v3/SELF_HOSTING.md` §2 (migration discipline)
- `feedback_corrections_must_grep_verify_source` discipline — verify substrate state before authoring changes
- `feedback_template_relocation_paper_shrink_discriminator` — codegen-driver must read .dag substrate, not Rust templates (applies once Phase 2 codegen-driver retirement starts; relevant context now)
