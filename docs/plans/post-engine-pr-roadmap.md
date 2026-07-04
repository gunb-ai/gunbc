# Post-engine-PR roadmap — deferred ledger + compiler algorithm audit

**Status:** authored 2026-07-04 at the tail of PR #6235 (constructor-owner ruling). Two parts: (A) everything the engine PR deliberately deferred, so nothing silently drops; (B) the plan for the operator's next directive — assess every compiler algorithm for rock-bottom (optimal) efficiency, generalizing the resolver lesson. Dissolves when: every §A item is either landed or has its own carrier doc, and §B's audit table lands with measured receipts and each SUSPICIOUS/PATHOLOGICAL row has a fix commit or a priced deferral.

DESIGN refs: §2 (the resolver lesson IS §2's redundancy axes), §5 (fail-open seams found on the way), §6 (price fixes by displaced cost, not elegance).

---

## A. Deferred ledger (engine PR #6235)

Ordered by weight, heaviest first.

1. **Emitter restoration → regen convergence (C8 tail).** Regen was semantically broken on main before this PR; three deficiencies fixed (statement-separator instances dissolved with the env counters; `needs_box_wrapping` shared-dominates reorder; `phase_profile.rs` registry), three open with receipts in `resolver-graph-major-design.md` §7b: deref-side boxing asymmetry (~800 errors — field-ACCESS emission has a second boxing-decision site that must consult the one `needs_box_wrapping` authority), alias-brand rendering (`Nat` emitted as its grounding `Rc<CommutativeSemiring<Magnitude>>`, ~250), misc residue (~80). Acceptance: regen output builds + full suite green + `bootstrap_fixed_point` stage1==stage2 + RegenVerifyGate green; `--emit-fresh` probes are ~2 min each post-pathology. Also fold in: the emitter's statement-then-expression mis-render (instances gone, deficiency remains); the seed hand-patches this PR accumulated (constructor ladder, chain followers, `resolve_generic_use_decl` parents-retry, cascade-locator message) all get replaced by true emission here.
2. **Emit-stage cost (the next resolver-class pathology).** `--target dag` emit of the 89-module wave-1 closure exceeds 20 minutes single-core (typecheck of the same closure: 21s); whole-tree emit unbounded in observation. Blocks the CI compile-clean gate's 10-minute budget. Profile first (extend the phase profiler into emit with per-module attribution), then fix; §B's survey carries the candidate mechanisms.
3. **Kernel-prelude shadowing — the principled root.** This PR's fix is surgical (a type-argument-bearing use site that resolves paramless retries direct import parents). The principled rule — explicit imports shadow the implicit kernel prelude everywhere, types AND functions, the same precedence the constructor kernel-merge already uses — was tried wholesale and broke kernel *function* dispatch (`join`/`split`/`length` "not found") through an unmapped path; that path must be mapped before the flip lands. Includes unifying the single-import vs multi-import ancestry asymmetry (`build_type_env` merges kernel as overlay only when imports ≠ 1).
4. **`Value::Null` split (DESIGN open thread, operator-fenced runway).** The kernel-optional raw-value-or-Null representation caused the shadowed-unwrap interpreter bug this PR fixed narrowly (guards hoisted). The split — Optional/Witness/miss into own carriers (~131 `present == None` sites) — remains the root fix; the hoisted guards and the `CrossRepresentationEquality` backstop dissolve with it.
5. **Flat visibility (rule ii) decision.** Operator: include if a re-census with the fixed compiler shows < ~300 sites needing explicit imports; the re-census never ran (the engine PR's tail went to the cascade root-cause instead). Cheap now: instrument `lookup_binding_by_name`'s ancestry rung with a hit counter + site log over a whole-tree compile, count distinct (file, name) pairs.
6. **C9 receipts (this PR's own tail, if not landed before merge):** full witness-suite run via claim_batch; `main_wet` generated-doc regen + drift gate; timing receipts into `resolver-graph-major-design.md` §0 placeholders (single-module before/after, whole-tree compile, floor phase marks vs the 10-min budget — note the pre-fix corpus baseline receipt: 4 hours, zero verdicts); release-binary rebuild (re-greens `interp_dry_run_test`); CI-green confirmation on #6235.
7. **Determinism roster re-signing (#5941).** `v2.std.determinism` rows `^unique_imported_variant_owner` and `^alpha_sorted_variant_fold` name fns this PR deleted; quoted symbols still compile but the roster is stale — operator re-signs successor rows (`insert_variant_owner_checked` / the declaration-ordered folds).
8. **Binder find-first hardening.** `owner_of_exported_arm` / `exported_coproduct_item` take the first matching coproduct within a module's items — ambiguity is unreachable in a swept corpus (the defining module collides first) but the walls should not rely on that indirection; make them collision-aware at regen time.
9. **Diagnostics polish.** Collision diagnostics print 2–3× per occurrence (dedup at the reporting seam); `is_user_generic_use_site`'s remaining kernel-name special cases (`is_container_type` gate) audited alongside §A.3.
10. **Witness re-enrollment (standing dissolve-on).** CI witness roster is minimal by design (opt-in inversion); enrollment returns to discovery shrunk by the affected set when `v2.lens.affected_set` selection + floor memoization land.
11. **Coverage-by-illusion census.** The shadowed-unwrap bug survived in an idiom used corpus-wide (`match xs |> first { Present … }` over records) because no green test ever executed that shape — a §5 concern: measure which corpus fns/branches the witness suite actually executes; the delta is the illusion surface.
12. **Resolver roadmap (design doc §7):** S2a runner + node-keyed store (CI floor first with unit results, byte-identical 11-gate receipt; then the module lane — per-module typecheck as antichains), S2b Merkle persistence (gated #5941), S3 shared store; `Rc`→`Arc` retires the per-shard W× residual.

## B. Compiler algorithm audit — "rock bottom" efficiency

**The resolver lesson, generalized.** Every stage cost factors as `units × work-per-unit`. The resolver failed on both axes at once: request-major scheduling multiplied *units* (every entry re-resolved the shared prefix) and the constructor scan blew up *work-per-unit* (per-literal whole-env flatten). Neither showed up as a diagnosis until instrumented — CI just timed out. The audit closes both axes for every stage, with measurement before and after.

**Method (each stage):**
1. **Inventory** — name every pass, its unit, its asymptotic class in named scaling variables (N modules, M nodes/module, E visible names, C closure, A arms/fields, D import depth), and its sharing story (computed once per what?).
2. **Verdict per pass** — OPTIMAL (information-theoretic floor: touches each input once) / ACCEPTABLE (log-linear) / SUSPICIOUS (polynomial where linear is plausible) / PATHOLOGICAL (measured blowup).
3. **Measure before fixing** — extend the phase profiler + per-stage attribution counters (the typecheck-attribution pattern) so every SUSPICIOUS verdict gets a corpus-scale number; the operator's standing "not very scientific" bar applies.
4. **Fix by displaced cost** (§6) — the 5ms pass doesn't get a pass for not being the 80s one, but fixes land in measured-share order; every fix carries a before/after receipt on the same fixture.
5. **Wall the floor** — each stage gains a budget witness (the floor phase marks pattern) so a future regression is a red gate, not a slow rot discovered at the 90-minute timeout.

**Survey inventory:** populated from the eight-stage parallel survey (parse/lex, resolve, infer-env, infer-core, emit, interpreter, floor-host, v2-interpreted stages) — table appended below when the fleet reports.

**Known entries ahead of the survey** (from this session's receipts): emit `--target dag` (PATHOLOGICAL, measured >20 min / 89 modules); per-shard union-index duplication (ACCEPTABLE-interim, W× bounded, S2b retires); interpreted v2 stages amplify constants ~100× so anything super-linear there is a priority multiplier.

## Survey results

_(appended when the fleet reports)_
