# gunbc Roadmap

Where the project is headed, at a glance. For *why* — the objective and the principles that protect
it — read [DESIGN.md](DESIGN.md). This is a deliberately thin list of high-level projects; the detailed
dependency graphs are being reformed and are not tracked here yet.

**v2 is the active phase.** v1 is the production self-hosted compiler (the `gunbc` CLI) and v2's seed;
v2 is the substrate-deep rewrite of the pipeline in [`src/v2/`](src/v2/). v3 was removed — its one
load-bearing role (the method-template projection producer) migrated into v1.

## High-level projects

- **Stage-fold** — every compiler stage collapses to one fold over its model
  (`stage = fold_carrier ∘ stage_algebra`); every former hand-arm becomes a data row. Nearly closed —
  `01_tokenize` (a `fold_source` combinator) is the last stage.
- **Control-flow bodies** — Branch → Bind → Loop via a COMPREP function-body producer. The pivot the
  next two projects sit on.
- **Emit breadth** — N data rows, not N×M hand-arms; includes the bidirectional emit/ingest round-trip
  (the fold's inverse proof).
- **Runnable v2 program with I/O** — effect handlers, run-loop/scheduler.
- **Self-hosting** — census ratchet toward zero hand-maintained Rust; the v1 seed is the last residual.
- **The Realization pattern** (cross-cutting, operator-elevated) — content-addressed reconciliation of
  a pure spec to its realized effect across the model→host boundary: one kernel, N handlers. The
  recurring root behind language-level caching, build caching, and the OS/provisioning work.
