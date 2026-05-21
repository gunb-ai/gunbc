# v4 Compile / Lens / Artifact Orthogonality Audit — 2026-05-21

**Scope:** audit `docs/design-v4-compiler-homomorphism.md` against the live v4 tree for three axes:
ground/observe/project/authorize split, compile-core vs lens/artifact ownership, and narrow artifact substrate vs full projection substrate.

**Method:** read the project invariants, then inspect:
`src/v4/compiler/00_compile.dag`, `04_infer.dag`, `05_emit.dag`,
`src/v4/lens/application.dag`, `src/v4/lens/registry.dag`,
`src/v4/std/artifact.dag`, and `src/v4/TASKS.md`.

## Findings

| Check | Live state | Disposition |
|---|---|---|
| Compile core signature | `00_compile.dag` declares `compile(source: CoreNode, input_lang: LanguageModel, mode: CompileMode) -> Outcome<CompileOutput>`. | Current scaffold conflict: `input_lang` and `CompileMode` are scalar knobs on compile. Target shape is `ground(IngestionPlan) -> observe -> project(ProjectionPlan) -> authorize`, with ingestion/projection requests using producer references and typed params. |
| Public lens gate | `validate_then_compile` runs `infer`, then `run_required_lens_gates`, then `compile_inferred`, returning `Validated<CompileOutput>`. Manual TestClaims assert empty-lens bypass and rejecting-lens block. | Current scaffold conflict in ownership: lenses are inputs to session policy, not compile-core. Keep `Validated<T>`, but reframe as terminal authorization over `ArtifactSet`. |
| Specific lens imports in core | `04_infer.dag` imports no `v4.lens.*` module. | Prior `v4.lens.cost.SymbolicCost` violation is resolved; design-doc stale row corrected. |
| Inferred facts | `InferredFacts` carries canonical grounding witness but not the full P6 semantic dependency graph. | Remaining T-9 gap kept in design-doc migration table. |
| Emit shape | `05_emit.dag` composes `translate` then `serialize_target`; serialization is still a fail-closed emitted-tree equality/source stub, not inverse grammar walk. | Shape aligned; implementation still staged until grammar-as-bidirectional-data serialization lands. |
| Artifact substrate | `std/artifact.dag` declares `ArtifactKind`, `Artifact`, and `NodeArtifactProvenance`. | Design-doc stale "not declared" corrected to "narrow declared"; full P8 regeneration substrate remains open. |
| Lens application surface | `lens/application.dag` owns `apply_lens(lens, SectionRef, Introspect/Enforce)` and the single advisory-to-fail-closed bridge. | Current scaffold conflict: `observe(inferred, LensPlan) -> LensReport`; move `Enforce` semantics to `authorize`. |
| Lens family shape | Cost/complexity headers say `Node -> Witness<...>`; effect/parallelism/idempotency helpers consume `InferredTree` plus dependency lists. | Mixed. Consumers should take the grounded graph (`InferredTree` / future `InferredGraph`) and return readings; distinguish proof witnesses from lens readings. |
| Graph vs subgraph | No literal `Subgraph` substrate type in the audited compiler/lens surface, but `affected_set` and TASKS prose use changed-subgraph frontier language. | Do not introduce one universal `SubgraphScope`. Use a low-level `RegionRef` and refine into `LensScope` / `ProjectionSubject` per consumer. |

## Boundary Statement

The target v4 shape preserves four separate outcomes:

- **Grounding:** `Outcome<InferredTree>`; mechanical compiler failure.
- **Lens report:** readings over the grounded graph.
- **Projection report:** independent per-artifact projection outcomes over the grounded graph.
- **Terminal authorization:** `Outcome<Validated<ArtifactSet>>`; policy over lens/projection results.

No code change was made for this audit. The source reshape is a follow-up: introduce the session four-phase split, replace `CompileMode` with a projection registry, reframe `Validated<T>` as terminal authorization, and move lens gating out of compile-core.
