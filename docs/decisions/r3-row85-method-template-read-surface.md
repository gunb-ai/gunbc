# R3 Row 85 MethodTemplateContract Read Surface

**Date:** 2026-05-03  
**Status:** decision artifact for PB #1560 Gap 4 routing  
**Scope:** documentation only; no PB hook, row edits, or carrier changes.

## Decision

The canonical read surface for `MethodTemplateContract` row facts consumed
outside the v3 std module graph is the committed full bootstrap `Dag` snapshot.

Concretely, the source of row text remains:

- `src/v3/std/rust_method_template_contracts.dag`
- `src/v3/std/python_method_template_contracts.dag`
- `src/v3/std/go_method_template_contracts.dag`

The consumable projection is the lowered `ValueBody::List` data declarations in
the full bootstrap snapshot generated from those files. Today that projection is
materialized by `src/v3/compiler/src/bootstrap_generated.rs` and read in-process
as `generated_full_bootstrap_dag()` / `Dag::new()`. A PB-owned build step may
wrap that projection for v2-retirement work, but the wrapper is a consumer hook,
not a second authority and not a copied row map.

## Typed Contract Ownership

Substrate + Grounding own the typed row contract:

- `src/v3/std/emit_model.dag` owns `MethodTemplateContract`.
- `src/v3/std/methods.dag` owns `MethodRef`.
- The three per-target row files own row membership and row text.

No new row carrier is needed for PB #1560 Gap 4. The existing
`MethodTemplateContract` product is sufficient for the current projection:
method identity, runtime template, emit template, wrapping policy, and
placeholder convention. `docs/audit/method-render-identity-6q.md` keeps
`MethodRef` as the current render-row identity and names the future trigger for
any richer render key.

PB-Bootstrap-Process owns the consumer hook location and scheduling. It must
consume the substrate projection above rather than define a PB-local schema or
duplicate template strings in v2/dsl sources.

## Non-Fork Ratchet

The projection cannot drift from the row authorities unless one of these gates
fails:

1. `regen_bootstrap --verify` proves the committed full bootstrap snapshot is
   freshly generated from the `.dag` authorities.
2. `method_template_contract_per_target_dag_method_unique` proves each target
   list resolves structurally by `MethodRef`.
3. `bootstrap_method_template_contracts_lower_to_list_with_empty_diagnostics`
   proves all three method-template-contract declarations are present in the
   full bootstrap snapshot, lower to `ValueBody::List`, and sit in a diagnostic-
   clean bootstrap Dag.
4. The row-82 `bootstrap_authority` carrier names the bootstrap membership set
   that includes the method-template row authority files. Verification should
   reuse that membership surface instead of hand-enumerating a parallel table.

Those gates make `bootstrap_generated.rs` a projection artifact. They do not
make generated Rust source the row authority.

Ratchet 1 may become vacuous or reshape under the #1558 dissolution-first
transition if `bootstrap_generated.rs` stops being committed and the full
bootstrap snapshot is materialized by `build.rs` or an `OUT_DIR` artifact. The
substrate of this decision is invariant to that reshape: the full bootstrap Dag
snapshot remains the canonical read surface regardless of whether its
materialization is committed Rust or an ephemeral build artifact.

## PB Surface Classification

The PB projection is both:

- **Build-step surface:** a PB-owned hook can read the committed full bootstrap
  projection to retire v2 legacy emit maps without importing `v3.std.*`.
- **Test-oracle surface:** tests may compare PB hook output against the same
  generated full bootstrap Dag projection.

Both uses must share the same substrate projection and ratchets. A test-only
oracle that is not the build-step source, or a build-step artifact without the
ratchet above, would reopen the parallel-authority gap.

## Dependency on Row 82

This decision depends on row 82 / `BootstrapAuthority` only for bootstrap
membership and diagnostics cleanliness. If row 82 changes the canonical
bootstrap membership surface, PB #1560 must update its consumer hook to follow
that surface.

It does not require a new method-template carrier and does not implement the
Verification `diagnostics_empty_after_bootstrap` claim. It closes only the
method-template-contract read-surface decision blocker for row 85 / PB #1560
Gap 4.

## Debt Receipt

This is the decision receipt for ROADMAP.md:512 and
`docs/debt/r3-debt-paydown-ledger-2026-05-02.md:85` as they apply to
MethodTemplateContract consumer migration:

- Row authorities remain the single source of row text.
- The committed full bootstrap Dag snapshot is the canonical structural
  projection outside the v3 std module graph.
- PB #1560 may implement a consumer hook against that projection, but must not
  create a copied map, direct `v3.std.*` import bridge, or alternate schema.

## Non-Goals

- No PB v2 hook implementation.
- No rewrite of `LanguageSpec.method_templates`.
- No edits to `src/v3/std/{rust,python,go}_method_template_contracts.dag`.
- No new `MethodTemplateContract` carrier.
