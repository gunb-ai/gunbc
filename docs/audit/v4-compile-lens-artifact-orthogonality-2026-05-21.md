# v4 Compile / Lens / Artifact Orthogonality Audit — 2026-05-21

**Scope:** audit `docs/design-v4-compiler-homomorphism.md` against the live v4 tree for three axes:
compile operations vs lens readings, compile operations vs artifact projection, and narrow artifact substrate vs full regeneration substrate.

**Method:** read the project invariants, then inspect:
`src/v4/compiler/00_compile.dag`, `04_infer.dag`, `05_emit.dag`,
`src/v4/lens/application.dag`, `src/v4/lens/registry.dag`,
`src/v4/std/artifact.dag`, and `src/v4/TASKS.md`.

## Findings

| Check | Live state | Disposition |
|---|---|---|
| Compile core signature | `00_compile.dag` declares `compile(source: CoreNode, input_lang: LanguageModel, mode: CompileMode) -> Outcome<CompileOutput>`. | Current scaffold conflict: `CompileMode` is a global mode enum. Target shape is separate `translate(node, target)` / `eval(node, host)` / artifact projection calls. |
| Public lens gate | `validate_then_compile` runs `infer`, then `run_required_lens_gates`, then `compile_inferred`, returning `Validated<CompileOutput>`. Manual TestClaims assert empty-lens bypass and rejecting-lens block. | Current scaffold conflict: lens output is data; caller/project policy gates on readings. `Validated<T>` and built-in rejecting-lens-as-compile-error are follow-up source reshapes. |
| Specific lens imports in core | `04_infer.dag` imports no `v4.lens.*` module. | Prior `v4.lens.cost.SymbolicCost` violation is resolved; design-doc stale row corrected. |
| Inferred facts | `InferredFacts` carries canonical grounding witness but not the full P6 semantic dependency graph. | Remaining T-9 gap kept in design-doc migration table. |
| Emit shape | `05_emit.dag` composes `translate` then `serialize_target`; serialization is still a fail-closed emitted-tree equality/source stub, not inverse grammar walk. | Shape aligned; implementation still staged until grammar-as-bidirectional-data serialization lands. |
| Artifact substrate | `std/artifact.dag` declares `ArtifactKind`, `Artifact`, and `NodeArtifactProvenance`. | Design-doc stale "not declared" corrected to "narrow declared"; full P8 regeneration substrate remains open. |
| Lens application surface | `lens/application.dag` owns `apply_lens(lens, SectionRef, Introspect/Enforce)` and the single advisory-to-fail-closed bridge. | Current scaffold conflict: collapse toward `Node -> Reading`; move enforcement to caller policy. |
| Lens family shape | Cost/complexity headers already say `Node -> Witness<...>`; effect/parallelism/idempotency helpers consume `InferredTree` plus dependency lists. | Mixed. Preserve arbitrary-node read direction; pressure-test whether `Witness` is a proof carrier or a reading shaped as a gate. |
| Whole graph vs part | No literal `Subgraph` substrate type in the audited compiler/lens surface, but `affected_set` and TASKS prose use changed-subgraph frontier language. | Do not introduce `Subgraph` / `SubgraphScope`; represent the region by the `Node` passed to the operation. |

## Boundary Statement

The target v4 shape preserves three separate authorities:

- **Compile operations:** independent Node-in calls such as `translate(node, target)` and `eval(node, host)`.
- **Lens readings:** independent Node-in calls that return typed readings. Policy may gate on those readings outside the lens.
- **Artifact projection:** independent projection/result production over Node. Narrow identity/provenance substrate exists, but regeneration authority remains incomplete until `Projection`, general `AffectedSet`, and `RecomputePlan` land as one coherent P8 cluster.

No code change was made for this audit. The source reshape is a follow-up: split public compile surfaces, retire `Validated<T>` lens gating, collapse `apply_lens` toward Node-in readings, and keep artifact generation out of compile results.
