# Method-Template Consumer Migration Audit

Status: Phase 1 audit for R3 Grounding method-template consumer migration.
Authority target: registry-backed `MethodTemplateContract` rows in
`src/v3/std/{rust,python,go}_method_template_contracts.dag`.

## Scope And Standing Debt

`ROADMAP.md:366` records that the triple `MethodTranslation` schema was retired
by PR #1210, and that the remaining emit-side authorities
(`rust_simple_method_specs`, derived `*_method_templates`, and
`rust_method_wraps_result`) still serve v2 emit until Pure-Bootstrap-Zero / v2
retirement can migrate those consumers.

`ROADMAP.md:512` elevates the remaining work: row population alone does not buy
the invariant while old runtime/emit tables still serve consumers. The R3 debt
ledger row `Method-template consumer migration`
(`docs/debt/r3-debt-paydown-ledger-2026-05-02.md:85`) keeps the debt open with
the acceptance check "Migrate consumers off old runtime/emit tables before
adding more rows."

## Replacement Authority Shape

`src/v3/std/emit_model.dag:488-494` declares:

```text
type MethodTemplateContract {
  dag_method: MethodRef
  runtime_template: String
  emit_template: MethodEmitTemplate
  wraps_result: Bool
  placeholder_convention: PlaceholderConvention
}
```

The replacement read is therefore target-specific row selection from:

| Target | Registry-backed rows | Notes |
| --- | --- | --- |
| Rust | `src/v3/std/rust_method_template_contracts.dag:83` `data rust_method_template_contracts: List<MethodTemplateContract>` | Header lines `:5-17` explicitly classify this as the v3-side future emission authority and explain the v2/PB-Zero deferral. Rows carry both `runtime_template` and `emit_template`; `wraps_result` replaces `rust_method_wraps_result`. |
| Python | `src/v3/std/python_method_template_contracts.dag:58` `data python_method_template_contracts: List<MethodTemplateContract>` | Header lines `:6-25` state that legacy `python_method_templates` lacks `wraps_result`; rows declare it explicitly false. `string_contains` remains a substrate classification gap because it is in the legacy map but not in `dsl/std/methods.dag`. |
| Go | `src/v3/std/go_method_template_contracts.dag:62` `data go_method_template_contracts: List<MethodTemplateContract>` | Header lines `:6-33` mirror the Python `wraps_result` handling and call out two gaps: `string_contains` lacks registry identity, and `chars` is skipped because the escaped empty-string runtime template currently fails structural lowering. |

A migrated consumer should look up a row by target row-list plus `dag_method:
MethodRef`, then use:

- `emit_template` instead of the legacy `method_templates: Map<String, String>`.
- `runtime_template` where a runtime-template read is needed.
- `wraps_result` instead of `rust_method_wraps_result()`.
- `placeholder_convention` instead of assuming all legacy map values use the
  named `{recv}` / `{arg}` convention.

## Consumer Inventory

`rg -n "method_templates|rust_method_wraps_result|rust_simple_method_specs"` at
HEAD finds the following live consumer classes after excluding documentation and
`src/v3/compiler/src/bootstrap_generated.rs`. Generated stage0 mirrors are
included because they are Rust consumers generated from the v2 `.dag` source
surface and must disappear with the source authority.

| Consumer | Old authority read | Current behavior | Replacement read / blocker |
| --- | --- | --- | --- |
| `dsl/extdeps/languages/rust/emit.dag:53-76` | Declares `rust_simple_method_specs`, derives `rust_method_templates()` and `rust_method_wraps_result()` from it. | Source old authority. `rust_method_templates()` feeds `LanguageSpec`; `rust_method_wraps_result()` feeds Rust Rc-wrapping decisions. | Delete after all v2 consumers read `rust_method_template_contracts`. The row list has equivalent `emit_template` and `wraps_result` for the simple methods (`src/v3/std/rust_method_template_contracts.dag:83+`). |
| `dsl/extdeps/languages/python/emit.dag:87-107` | Declares `python_method_templates: Map<String, String>`. | Source old authority for Python method rendering. | Delete after consumers read `python_method_template_contracts`. Blocker for complete parity: `string_contains` is present in the legacy map but not in the method registry (`src/v3/std/python_method_template_contracts.dag:12-18`). |
| `dsl/extdeps/languages/go/emit.dag:83-99` | Declares `go_method_templates: Map<String, String>`. | Source old authority for Go method rendering. | Delete after consumers read `go_method_template_contracts`. Blockers for complete parity: `string_contains` lacks registry identity and `chars` row population is skipped by the current tokenizer/lowering issue (`src/v3/std/go_method_template_contracts.dag:12-29`). |
| `src/v2/languages.dag:35-54` | Imports `rust_method_templates`. | Carries Rust legacy templates into `LanguageSpec`. | Replace `LanguageSpec.method_templates` with a registry-row projection once v2 can consume bootstrap Dag rows. PB-Zero blocks that today. |
| `src/v2/languages.dag:56-73` | Imports `python_method_templates`. | Carries Python legacy templates into `LanguageSpec`. | Same projection requirement; also blocked by `string_contains` registry classification for full deletion. |
| `src/v2/languages.dag:75-90` | Imports `go_method_templates`. | Carries Go legacy templates into `LanguageSpec`. | Same projection requirement; also blocked by Go `string_contains` / `chars` gaps for full deletion. |
| `src/v2/languages.dag:390-400` | Defines `LanguageSpec.method_templates: Map<String, String>?`. | The v2 target abstraction is map-shaped, so it cannot carry `MethodRef`, `runtime_template`, `emit_template`, `wraps_result`, or `placeholder_convention`. | Structural rewrite: replace or supplement this field with a typed `MethodTemplateContract` projection. PB-Zero/bootstrap-Dag-consumer infrastructure is the gating prerequisite. |
| `src/v2/languages.dag:544` | Sets Rust spec `method_templates: rust_method_templates()`. | Rust and Rust-backed test target read the old map. | Read `rust_method_template_contracts` rows and project `emit_template` by `MethodRef`; blocked by v2 row-consumer infrastructure. |
| `src/v2/languages.dag:688` | Sets Python spec `method_templates: python_method_templates`. | Python target reads the old map. | Read `python_method_template_contracts`; blocked by v2 row-consumer infrastructure plus `string_contains` classification for complete deletion. |
| `src/v2/languages.dag:832` | Sets Go spec `method_templates: go_method_templates`. | Go target reads the old map. | Read `go_method_template_contracts`; blocked by v2 row-consumer infrastructure plus Go row gaps. |
| `src/v2/languages.dag:979` | Sets `RustTest` spec `method_templates: rust_method_templates()`. | Test target duplicates the Rust map authority. | Same as Rust spec migration. |
| `src/v2/05_emit.dag:2570-2581` | Reads `language_spec(target).method_templates` and applies map values via `apply_named_template`. | Shared algebra method template dispatch for non-Rust-specialized emit. | Replace map lookup by `MethodRef` row lookup against the target row-list, then apply `emit_template` under `placeholder_convention`. Blocked by `LanguageSpec` shape and bootstrap-Dag row access. |
| `src/v2/05_emit_rust.dag:66-71` | Imports `rust_method_wraps_result` from `extdeps.languages.rust.emit`. | Makes the Rust-only wrapping bit available to the Rust emitter. | Delete import once Rust wrapping decisions read row `wraps_result`. |
| `src/v2/05_emit_rust.dag:2436-2441` | Calls `map_contains_key(rust_method_wraps_result(), function_name)`. | Decides whether Rust collection results need `Rc::new(...)`. | Replace with lookup of the selected Rust `MethodTemplateContract.wraps_result`; blocked by row lookup from the Rust emit path. |
| `src/v2/05_emit_rust.dag:3486-3494` | Reads `language_spec(Rust).method_templates`; then separately asks `rust_method_wraps_result`. | Rust method emit is split across two old authorities: template map and wrapping map. | Single row read should provide both `emit_template` and `wraps_result`; blocked by v2 row-consumer infrastructure. |
| `src/v2/stage0/src/extdeps_languages_rust_emit.rs:51-82` | Generated Rust for `rust_simple_method_specs`, `rust_method_templates`, `rust_method_wraps_result`. | Generated mirror of `dsl/extdeps/languages/rust/emit.dag`; do not edit directly. | Regenerated deletion after the source `.dag` authority retires. |
| `src/v2/stage0/src/extdeps_languages_python_emit.rs:206-230` | Generated Rust for `python_method_templates`. | Generated mirror of `dsl/extdeps/languages/python/emit.dag`; do not edit directly. | Regenerated deletion after the source `.dag` authority retires. |
| `src/v2/stage0/src/extdeps_languages_go_emit.rs:188-210` | Generated Rust for `go_method_templates`. | Generated mirror of `dsl/extdeps/languages/go/emit.dag`; do not edit directly. | Regenerated deletion after the source `.dag` authority retires. |
| `src/v2/stage0/src/v2_compiler_languages.rs:23-55` | Imports generated `go_method_templates`, `python_method_templates`, `rust_method_templates`. | Generated mirror of `src/v2/languages.dag` imports. | Regenerated after `src/v2/languages.dag` migrates. |
| `src/v2/stage0/src/v2_compiler_languages.rs:379` | Defines generated `LanguageSpec.method_templates`. | Generated map-shaped target abstraction. | Regenerated after source `LanguageSpec` shape migrates. |
| `src/v2/stage0/src/v2_compiler_languages.rs:522,703,886,1053` | Assigns generated target specs from old maps. | Generated mirrors of `src/v2/languages.dag:544,688,832,979`. | Regenerated after source target specs migrate. |
| `src/v2/stage0/src/v2_compiler_emit.rs:4861-4882` | Reads generated `LanguageSpec.method_templates`. | Generated mirror of `src/v2/05_emit.dag:2570-2581`. | Regenerated after shared emit migrates to row lookup. |
| `src/v2/stage0/src/v2_compiler_emit_rust.rs:4-8` | Imports generated `rust_method_wraps_result`. | Generated mirror of `src/v2/05_emit_rust.dag:66-71`. | Regenerated after Rust emit migrates. |
| `src/v2/stage0/src/v2_compiler_emit_rust.rs:5245-5248` | Calls generated `rust_method_wraps_result()`. | Generated mirror of `src/v2/05_emit_rust.dag:2436-2441`. | Regenerated after Rust wrapping reads row `wraps_result`. |
| `src/v2/stage0/src/v2_compiler_emit_rust.rs:8518-8526` | Reads generated `LanguageSpec.method_templates` and then Rust wrap map. | Generated mirror of `src/v2/05_emit_rust.dag:3486-3494`. | Regenerated after Rust method emit reads one row. |
| `src/v2/tests/src/source_audit.rs:10-21` | Names old authorities in `LEGACY_METHOD_TEMPLATE_AUTHORITIES` and allow-lists `src/v2/`. | Existing source-level deferral ratchet. | Keep until migration completes; shrink allow-list as source consumers retire. |
| `src/v2/tests/src/source_audit.rs:327-348` | Synthetic test proves non-v2 old-authority readers trip the ratchet. | Guards against new v3-side consumers of old tables. | Keep through Phase 2; retire when old authorities are deleted. |
| `src/v2/tests/src/source_audit.rs:1217-1285` | Tests `rust_method_wraps_result()` and `rust_method_templates()` derive exactly from `rust_simple_method_specs()`. | Ratchets internal consistency of the old Rust authority while it is still live. | Delete with `rust_simple_method_specs` / derived functions. Replacement test should assert Rust row `wraps_result` and `emit_template` parity at the row authority, not old-map derivation. |

## Substrate Gaps Blocking Complete Migration

1. **v2/PB-Zero row-consumer infrastructure.** All live production consumers are
   in `src/v2/` or generated from `src/v2/`. `src/v3/std/rust_method_template_contracts.dag:7-17`
   states the operational blocker: v2 emit cannot read bootstrap Dag rows today.
   The existing deferral ratchet in `src/v2/tests/src/source_audit.rs:295-325`
   enforces that no new non-v2 consumer joins the old-authority set.

2. **Map-shaped `LanguageSpec` cannot represent the registry row.**
   `src/v2/languages.dag:400` carries only `Map<String, String>?`, losing
   `MethodRef`, dual runtime/emit templates, `wraps_result`, and
   `placeholder_convention`. Migration is a structural rewrite, not a map
   source substitution.

3. **Target-only method classification remains open.** `string_contains` appears
   in legacy Python and Go maps (`dsl/extdeps/languages/python/emit.dag:95`,
   `dsl/extdeps/languages/go/emit.dag:87`) but is not in the method registry, so
   there is no `MethodRef` row key yet (`src/v3/std/python_method_template_contracts.dag:12-18`,
   `src/v3/std/go_method_template_contracts.dag:12-17`).

4. **Go `chars` row is still skipped.** The Go contract file documents that
   `chars` is present in runtime translations but skipped in Phase 1 because
   the escaped empty-string literal fails structural lowering
   (`src/v3/std/go_method_template_contracts.dag:20-29`). Complete old-authority
   deletion needs that row or an intentional deletion of the unsupported old
   behavior.

5. **Contract diagnostic gate is adjacent and still open.** `ROADMAP.md:502`
   records a live `go_method_template_contracts` diagnostic mismatch, and
   `ROADMAP.md:504` generalizes the missing "diagnostics empty" acceptance gate.
   That is Substrate/Verification-owned, but a production consumer should not
   treat row population as fully green until the row-list bootstrap diagnostic
   state is ratcheted.

## Retirement Sequence

1. **Keep the deferral ratchet live.** `src/v2/tests/src/source_audit.rs` already
   fail-closes on any new non-v2 old-authority reader. No new bridge or marker
   is needed for Phase 1.

2. **Resolve row parity gaps before deleting source authorities.** The substrate
   owner must classify or model Python/Go `string_contains`, resolve Go `chars`,
   and close the method-template-contract diagnostics-empty gate.

3. **Add a v2 bootstrap-Dag row projection.** This is the PB-Zero enabling slice:
   v2 emit needs a generated or otherwise structural projection from
   `{rust,python,go}_method_template_contracts` into the emitter's target
   context. The projection should preserve `MethodRef`, `emit_template`,
   `wraps_result`, and `placeholder_convention`; it should not re-create a
   second string-keyed map authority.

4. **Migrate leaf emit consumers.** Replace `src/v2/05_emit.dag:2570-2581` and
   `src/v2/05_emit_rust.dag:2436-2441,3486-3494` with row lookups. Rust should
   become a single selected-row read instead of split template/wrap maps.

5. **Migrate `LanguageSpec` shape.** Remove `method_templates:
   Map<String, String>?` from `src/v2/languages.dag:400` or replace it with the
   typed row projection. Update target spec construction at
   `src/v2/languages.dag:544,688,832,979`.

6. **Delete old source authorities.** Delete `rust_simple_method_specs`,
   `rust_method_templates()`, and `rust_method_wraps_result()` from
   `dsl/extdeps/languages/rust/emit.dag:47-76`; delete
   `python_method_templates` from `dsl/extdeps/languages/python/emit.dag:87-107`;
   delete `go_method_templates` from `dsl/extdeps/languages/go/emit.dag:83-99`.
   Regenerate stage0 so generated Rust mirrors disappear.

7. **Retire old-authority tests and shrink the ratchet.** Delete the Rust
   derivation tests at `src/v2/tests/src/source_audit.rs:1217-1285` when their
   subject functions disappear. The broad non-v2 ratchet can retire only when
   the old authority names no longer exist outside docs/history.

## Closing PR Shapes

| Slice | PR shape |
| --- | --- |
| Row parity gap closure | Substrate PR(s): add/classify missing `MethodRef` identity for `string_contains`; unblock Go `chars` row or explicitly remove that legacy behavior; close the diagnostics-empty gate for all three row lists. |
| PB-Zero row projection | PB/v2 infrastructure PR: v2 emitter can consume bootstrap Dag row data without a parallel string map. This is the enabling dependency for any real production migration. |
| Shared emit migration | Grounding PR: `src/v2/05_emit.dag` uses the typed projection and `MethodTemplateContract.emit_template`; generated `v2_compiler_emit.rs` updates by regen. |
| Rust wrap/template unification | Grounding PR: `src/v2/05_emit_rust.dag` reads one Rust row for `emit_template` and `wraps_result`; generated `v2_compiler_emit_rust.rs` updates by regen. |
| LanguageSpec shape cleanup | Grounding/PB PR: remove map-shaped `LanguageSpec.method_templates` and target assignments from `src/v2/languages.dag`; generated `v2_compiler_languages.rs` updates by regen. |
| Authority deletion | Grounding PR: delete old declarations/functions from `dsl/extdeps/languages/{rust,python,go}/emit.dag`; regenerate stage0; remove old derivation tests; update ROADMAP/ledger row to retired or partial as appropriate. |

## Phase 1 Conclusion

No v3-side production consumer remains to migrate today. The production readers
are v2 source and generated stage0 mirrors, behind the same PB-Zero
bootstrap-Dag-consumer wall named in the existing contract-file headers and the
existing source-level ratchet. The next executable retirement work is not more
row population by Grounding; it is row-parity gap closure plus the PB-Zero
consumer projection that lets v2 emit select `MethodTemplateContract` rows
structurally.
