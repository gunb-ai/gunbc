# v4 Compile / Lens / Artifact Orthogonality Audit — 2026-05-21

**Scope:** audit `docs/design-v4-compiler-homomorphism.md` against the live v4 tree for three axes:
compile-core vs lens enforcement, compile-core vs artifact projection, and narrow artifact substrate vs full regeneration substrate.

**Method:** read the project invariants, then inspect:
`src/v4/compiler/00_compile.dag`, `04_infer.dag`, `05_emit.dag`,
`src/v4/lens/application.dag`, `src/v4/lens/registry.dag`,
`src/v4/std/artifact.dag`, and `src/v4/TASKS.md`.

## Findings

| Check | Live state | Disposition |
|---|---|---|
| Compile core signature | `00_compile.dag` declares `compile(source: CoreNode, input_lang: LanguageModel, mode: CompileMode) -> Outcome<CompileOutput>`. | Design-doc stale row corrected. Core is data-in/data-out. |
| Public lens gate | `validate_then_compile` runs `infer`, then `run_required_lens_gates`, then `compile_inferred`, returning `Validated<CompileOutput>`. | Orthogonality clarified: lens enforcement gates the terminal before homomorphism invocation; lens outputs are not translate/eval inputs. |
| Specific lens imports in core | `04_infer.dag` imports no `v4.lens.*` module. | Prior `v4.lens.cost.SymbolicCost` violation is resolved; design-doc stale row corrected. |
| Inferred facts | `InferredFacts` carries canonical grounding witness but not the full P6 semantic dependency graph. | Remaining T-9 gap kept in design-doc migration table. |
| Emit shape | `05_emit.dag` composes `translate` then `serialize_target`; serialization is still a fail-closed emitted-tree equality/source stub, not inverse grammar walk. | Shape aligned; implementation still staged until grammar-as-bidirectional-data serialization lands. |
| Artifact substrate | `std/artifact.dag` declares `ArtifactKind`, `Artifact`, and `NodeArtifactProvenance`. | Design-doc stale "not declared" corrected to "narrow declared"; full P8 regeneration substrate remains open. |
| Lens application surface | `lens/application.dag` owns `apply_lens` / `apply_diff` as the lens/read-edit surface and single advisory-to-fail-closed bridge. | Consistent with P3 orthogonality: application is a side-channel/gate, not a compile mode. |

## Boundary Statement

The live v4 shape preserves three separate authorities:

- **Compile core:** `CoreNode + LanguageModel + CompileMode -> Outcome<CompileOutput>`.
- **Lens enforcement:** fold/gate over `InferredTree`, surfaced publicly through `Validated<CompileOutput>`.
- **Artifact projection:** narrow identity/provenance substrate exists, but regeneration authority remains incomplete until `Projection`, general `AffectedSet`, and `RecomputePlan` land as one coherent P8 cluster.

No code change was needed for this audit. The patch is a design-doc reconciliation so future implementers do not chase already-resolved stale rows or overclaim artifact/regeneration completion.
