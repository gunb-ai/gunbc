# gunbc — Roadmap

`DESIGN.md` is the authority for *why*; this is *what's in flight*. A task's real state is its
branch/PR and the marks on the carrier (SCAFFOLD / dissolve-on / 🟡), not this file.

## 1. Session dashboard on `.dag`

Backend only — NOT the frontend.

- idea → PR pipeline

## 2. idea → idea compiler

Stop anchoring on code.

## 3. Self-hosting v2 (TypeScript)

End goal: v2 compiles its own `src/v2` and emits TypeScript to a bit-identical fixed point; the Rust
v1 seed shrinks toward zero. Today: **not achieved** — only the comparison machinery is green (on
fixtures); the whole-compiler fixed point is "Stage C", gated on candidate generation.

Plan (current state + stages + open questions): [docs/plans/v2-self-hosting.md](docs/plans/v2-self-hosting.md)

### De-fork dsl ↔ v2 (collapse the duplication)

`dsl/` is the single authority; `src/v2/` is the compiler. During bootstrap v2 made **mirror
copies** of pieces of `dsl/std` (because it can't import `dsl/` across trees yet). End goal:
**v2 imports `dsl/` directly, every duplicate collapses, and no concept has two homes** —
especially no genuinely historical fork (a name shared by two copies, or two names for one
concept). Folder/module naming should reflect one authority, not the fork's history.

- **Prerequisite (blocking):** turn on cross-tree import — it's wired but fail-closed
  (`src/v2/compiler/03_name_resolve.dag` defaults `FundamentalityUnknown` → deny). Needs
  grounded `source_root` tagging at ingest.
- Then collapse the clear duplicates (algebra, logic `Classical`→`Bool`, nat, reducible, measure).
- Then decide per-concept the same-name/different-job pairs (integer, effects, float, coercion,
  node, verification): rename to disambiguate or merge.

Audit (carrier-grounded census + sequencing): [docs/plans/dsl-v2-defork-audit.md](docs/plans/dsl-v2-defork-audit.md)

## 4. HTML / React rendering

Get it working.

## 5. Compute fabric

- privacy — done
- repo model (internal repo) on compute fabric
- CI on compute fabric
