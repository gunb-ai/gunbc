# T-Lens-Application-Surface Readiness Receipt

**Date:** 2026-05-02  
**Lane:** T-Lens-Application-Surface  
**Scope:** design/audit receipt only. No substrate or compiler implementation.

## Authority

- `docs/r3-structure.md` lane row for T-Lens-Application-Surface.
- `docs/design-lens-application-surface.md`.
- Existing substrate surfaces:
  - `src/v3/std/lens.dag`
  - `src/v3/std/dimensions.dag`
  - `src/v3/std/computation.dag`
  - `src/v3/lenses/{complexity,cost,parallelism}.dag`

## Verdict

The requested `apply_lens(lens, section, config)` surface already has a landed
design authority in `docs/design-lens-application-surface.md`. The clean
substrate shape is:

```dag
type SectionRef
  = DeclarationScope { declaration: DeclarationId }
  | NodeScope { declaration: DeclarationId, node: NodeId }

type ApplicationConfig
  = Enforce { budget: LensBudget, diagnostic_severity: DiagnosticSeverity }
  | Introspect

type DiagnosticSeverity = Error

type SectionedLensApplication {
  lens: DeclarationId
  section: SectionRef
  config: ApplicationConfig
  span: SourceSpan
}
```

This receipt should not land carriers yet. The design doc's cascade gate is
load-bearing: T-Lens-Behavioral-Parity must make the target lenses behaviorally
complete before application-surface enforcement can be honest. Landing carriers
now would be shape-only substrate without a complete lens consumer.

## Existing Surface Audit

`src/v3/std/lens.dag` already declares the generic `Lens<C>` primitive:

- `read: fn(Dag, Behavior) -> Witness<C>`
- `sequential: Monoid<C>`
- `branch: fn(C, C) -> C`
- `iterate: fn(C, LoopBound) -> C`
- `validate: fn(Dag, C) -> OptionalDiagnostic`

That shape is sufficient for a future application fold. The new surface should
not hand-Rust a second generic fold or fake a lens instance.

`src/v3/std/dimensions.dag` already carries `Witness<C>`,
`OptionalDiagnostic`, and `DimensionReport<C>`. These should be reused by
`Lens<C>` applications rather than reauthoring pass/fail result carriers.

`src/v3/std/computation.dag` carries the loop and bound vocabulary consumed by
the `Lens<C>.iterate` arm. Opt-in cross-iteration parallelism should reference
the real parallelism lens once T-Lens-Behavioral-Parity completes; it should not
invent a parallel node or a special compiler-side heuristic.

The existing lens implementations are not yet complete enough to support the
worked paths as enforcement:

- `src/v3/lenses/complexity.dag` still needs the behavioral-complete complexity
  substrate before complexity-contract compile errors can be meaningful.
- `src/v3/lenses/cost.dag` needs the cost basis rows used by CRDT and
  memory-peak examples.
- `src/v3/lenses/parallelism.dag` is still a fail-closed placeholder around the
  native Rust parallelism analysis, so opt-in iteration parallelism cannot be
  represented as a first-class lens application yet.

## Violation Policy

The user-facing dispatch names `CompileError | Warning | Silent`. The landed
design authority resolves this under fail-closed discipline:

- `CompileError` maps to `Enforce { ..., diagnostic_severity: Error }`.
- `Warning` is not admitted as a steady-state policy.
- `Silent` is not admitted.
- Non-enforcing use is `Introspect`, which computes the lens value without a
  budget or diagnostic.

This is the right state-space shape: enforcement cannot exist without a budget,
and introspection cannot accidentally carry a stale budget.

## Worked-Path Requirements

The first implementation slices must preserve these requirements:

| Worked path | Required substrate / consumer behavior |
|---|---|
| Complexity-contract compile error | `SectionedLensApplication` over a function declaration with `ApplicationConfig::Enforce`; violation routes to a compile-time diagnostic, not warning/silent. |
| CRDT cost basis | Cost lens budget remains owned by the cost lens authority; no central `LensBudget` roster. |
| Memory-peak cost basis | Memory dimension is a cost-lens budget/value shape, not a separate application mechanism. |
| Opt-in cross-iteration parallelism | Application targets a `NodeScope` loop node and uses the parallelism lens; no heuristic auto-parallelization. |

## Next Slice

After T-Lens-Behavioral-Parity is complete, the first clean implementation slice
is carrier-only:

1. Add `src/v3/std/lens_application.dag`.
2. Declare `SectionRef`, `ApplicationConfig`, `DiagnosticSeverity`, and
   `SectionedLensApplication` per `docs/design-lens-application-surface.md` §2.
3. Add focused substrate ratchets proving:
   - `SectionRef` is exactly `DeclarationScope | NodeScope`.
   - `NodeScope` carries both declaration and node context.
   - `ApplicationConfig` is exactly `Enforce | Introspect`.
   - `DiagnosticSeverity` admits only `Error`.
4. Regenerate bootstrap and parse manifest.

Parser syntax for `apply_lens(...)`, budget compatibility checking, duplicate
`(lens, section)` rejection, default complexity-policy synthesis, and the four
worked demonstrations should remain follow-on slices. They need real
behaviorally-complete lenses and should not be faked in this receipt.

## STOP Boundary

This audit stops short of implementation because the repo already contains the
design authority and the design names a real cascade gate. Implementing the
surface now would either create unused substrate or require fake lens behavior.
The honest next action is to wait for T-Lens-Behavioral-Parity completion, then
land the carrier-only slice above.
