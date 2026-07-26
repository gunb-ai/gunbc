# E0277 canonical-seven census (2026-07-26)

**Assignment:** parent (sharp-bee-290) reassigned E0599 to a dedicated session (swift-bee-52) and handed this session E0277 (603 canonical-seven occurrences, third UNOWNED bucket, `emit_host`=129 flagged as the likely-root outlier). Census-first, same method as E0599: diagnose before fixing.

**Baseline corroboration:** per-module `error[E0277]:N` counts in this run (76 / 75 / 81 / 76 / 129 / 76 / 90 across 06_translate / 04_infer / 05_eval / 05_emit / emit_host / emit_module / materialization_carriers) match `docs/probes/refresh_canonical_seven_2026-07-26.tsv` (git_sha `efe67794c`, #7275) **exactly**, at a later git_sha (`0e14c301b4`) with fresh local binaries (mtime/sha256 verified before running, per standing standard). No drift.

**Method:** `PROBE_KEEP_LOG_DIR` (added to `curated_cargo_probe_one.sh`, PR #7280) persists the raw `cargo build` stderr per module; census extracted via `grep '^error\[E0277\]: the trait bound' | sed -E 's/^error\[E0277\]: the trait bound `(.*)` is not satisfied$/\1/' | sort | uniq -c`, keyed by (unsatisfied trait bound, offending type) per the parent's spec. Full data: `docs/probes/e0277_trait_bound_census_2026-07-26.tsv`.

## Three distinct families, not one root

The `the trait bound X is not satisfied` pattern covers 79-93% of each module's E0277 total (581 of 603 tree-wide when the six 76/81-count modules are counted against `materialization_carriers`' residual below) and decomposes cleanly into three families that do **not** share a fix:

1. **Generic-type-parameter `Clone` (the dominant family, ~26-30 occurrences/module, present in every module).** `T: Clone`, `U: Clone`, `A: Clone`, `B: Clone` — bare, unbounded generic type parameters used at emit sites that require `Clone` but carry no `where T: Clone` bound in the emitted Rust signature.
   - **This is very likely the E0599 root, viewed from a different call shape.** The E0599 partial census (handed off to swift-bee-52) found the dominant pattern there was `no method named \`clone\` found for type parameter \`T\`/\`R\`/\`A\`` — same type-parameter letters (T/U/A/B/R), same missing-Clone defect. E0599 fires when the generic is used as a bare receiver (`x.clone()` with no method resolution possible at all); E0277 fires when the call *does* resolve (often via an explicit `Clone::clone(&x)` or a trait-object/where-clause context) but the bound isn't declared. Same underlying gap: **the emitter does not propagate a `Clone` bound onto generic type parameters that need it**, just observed through two different rustc diagnostic codes depending on how the generic is consumed at the use site.
   - This is the family to root-cause first before any fix slice — it's the shared root with E0599, and fixing it once should shrink both buckets simultaneously across all seven modules.

2. **`Node`/`EnvironmentBindingKey` missing `Hash`/`Eq` (5-6 occurrences/module, present in every module).** `Node: std::hash::Hash`, `Node: Eq`, `EnvironmentBindingKey: std::hash::Hash` — these carriers are used as map/set keys (`HashMap<Node, _>` or similar) in emitted code without the emitter having verified or derived `Hash`/`Eq` on the source type. Distinct from family 1 — no correlation with E0599's clone/is_empty/iter pattern. Likely a missing-derive gap at the struct/enum declaration → emit boundary for these two specific carrier types.

3. **`serde::Serialize`/`Deserialize` missing on interpreter/carrier structs (present in every module at a fixed ~3:1 Deserialize:Serialize ratio; `emit_host` and `materialization_carriers` additionally carry a large `CommutativeSemiring<Magnitude>` sub-family).** `ValueInterpreter`, `TransformInterpreter`, `MatchInterpreter`, `LoopInterpreter`, `BranchInterpreter`, `BindInterpreter`, `EffectIoEvalBundle` (05_eval/emit_host only) each miss `serde::Deserialize<'de>` (3x) and `serde::Serialize` (1x) — the 3:1 ratio suggests three call sites deserialize a value of that type per one that serializes it (not obviously a bug in itself, just the call-shape ratio at those particular carriers). `CommutativeSemiring<Magnitude>` is the single largest contributor tree-wide: 30 (Deserialize) + 6 (Serialize) in `emit_host`, 45 (Deserialize) + 9 (Serialize) in `materialization_carriers` — **this alone explains most of emit_host's and materialization_carriers' outlier totals** (129 vs the 76 baseline, and 90 respectively). Distinct root from families 1 and 2: a missing `#[derive(Serialize, Deserialize)]` on `CommutativeSemiring<Magnitude>` and the six/seven `*Interpreter`/`EffectIoEvalBundle` structs.

## Residual (materialization_carriers only, 11 of 90 E0277 occurrences)

Not covered by the `the trait bound X is not satisfied` first-line pattern (different rustc message shape), all in the same family-3 (`CommutativeSemiring<Magnitude>`) or unrelated:
- 9x ``CommutativeSemiring<Magnitude>` doesn't implement `Debug`` — same missing-derive family as #3 above (add `Debug` to the same derive fix).
- 1x `cannot add \`Rc<CommutativeSemiring<Magnitude>>\` to \`i64\`` — unrelated (an arithmetic-op trait bound on a different pairing, needs separate diagnosis, not part of any family above).
- 1x `can't compare \`std::string::String\` with \`fn() -> std::string::String {parse_table_memo_id}\`` — unrelated, a comparison against a bare function-item type where a call result was intended; likely an emitter bug at a call-site (missing `()`), not a trait-derivation gap at all.

## emit_host / materialization_carriers outlier explained

Parent flagged `emit_host`'s 129 (vs 76 baseline) as the likely-root outlier. This census shows it is **not** a different root — it's family 1+2+3 (same as every other module) **plus** the `CommutativeSemiring<Magnitude>` serde sub-family (36 occurrences) which only reaches `emit_host` and `materialization_carriers` (the two modules where that carrier type is actually referenced in emitted code). `materialization_carriers`' 90 (vs an implied ~76 baseline for a module of its size) is explained the same way, at an even larger share (54 of 90 = 60%) since `CommutativeSemiring<Magnitude>` is central to that module and families 1-2's `Node`/`EnvironmentBindingKey`/`*Interpreter` carriers barely appear there at all (module-appropriate — it doesn't touch those types).

## Coordination with swift-bee-52 (E0599)

Handed off partial E0599 census (2 of 7 modules, raw logs + method histogram) before starting this lane — see dashboard message `msg_8c8012fd`. Family 1 above (generic `T/U/A/B: Clone`) is very likely the same defect underlying E0599's dominant `clone`/`is_empty`/`iter`-on-type-parameter pattern (63/93 = 68% and 58/88 = 66% of E0599 in the two modules probed there). **Before either session commits to a fix**, compare: does swift-bee-52's E0599 census also show the same T/U/A/B letter distribution and the same im::Vector<T>/Rc<im::Vector<T>> carrier shapes? If so this is one root (a Clone-bound-propagation gap at generic emit sites) worth one fix, landed by whichever session gets there first with the other session reviewing.

## Next steps (not yet started)

1. Confirm family-1 root hypothesis against swift-bee-52's E0599 findings (coordination, above).
2. Locate the emitter site responsible for generic type-parameter bound propagation (likely in `05_emit.dag`/`emit_module.dag`'s type-expression-to-Rust-generics projection) — read before proposing a fix.
3. Propose a fix slice with a measured before/after target (families 1-3 are independently fixable; family 1 first since it's shared with E0599 and is the largest single bucket in every module).
4. Discriminating RED: a synthetic fixture with a generic function using a bound-requiring method on an unbounded type parameter, verified to still refuse post-fix if the bound is genuinely absent from the *source* (never fabricate a bound the .dag model doesn't declare).
