# Retro

## README.md invariant tracking

- Fixed: invariant 1 layout wording no longer points at `src/daglang/`; the root README now describes the numbered pipeline folders.
- Fixed: invariant 2 no longer names the nonexistent `src/lib/transport/`; it now points at `src/8_materialize/transport/`.
- Fixed: invariant 4 compile receipts are seeded from the already-loaded `ModuleGraph` source text instead of rereading files during receipt generation.
- Fixed: invariant 4 exec-runtime workspace path discovery moved out of `daglang-emit` and into `daglang-driver` preparation, so the emit stage is render-only.
- Fixed: invariant 6 no longer points at the nonexistent `src/gunbc-app/`; the README now describes `src/8_materialize/` as the runtime wiring layer.
