# gunbc — Roadmap

`DESIGN.md` is the authority for *why*; this is *what's in flight*. A task's real state is its
branch/PR and the marks on the carrier (SCAFFOLD / dissolve-on / 🟡), not this file.

## 1. Session dashboard on `.dag`

Backend only — NOT the frontend.

- idea → PR pipeline

## 2. idea → idea compiler

Stop anchoring on code.

## 3. Self-hosting v2 (TypeScript)

End goal: v2 compiles its own `src/v2` to a bit-identical fixed point; the Rust v1 seed shrinks toward
zero. Front-end is **done** (whole tree parses/resolves/infers); the active self-host path is **Rust**
(`--target rust`, cargo-green, regen_stage0) — TypeScript is one of 14 emit targets, not yet the
self-host runtime (open decision). Open last mile: the emitted whole-tree crate doesn't `cargo build`
green yet, and the fixed-point gate is still fail-closed (Stage C, gated on candidate generation).

Plan (verified state + the two tracks + open questions): [docs/plans/v2-self-hosting.md](docs/plans/v2-self-hosting.md)

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

## 6. Complexity / synthesis lens over the whole codebase

End goal: every `fn`/node carries a structural **complexity budget** the lens gates on — not just a
curated roster. Today coverage is partial:

- Complexity lens (`src/v2/lens/complexity.dag`) is the asymptotic projection of `cost.dag`
  SymbolicCost; it gates only a **subject roster** (`complexity_gate/subject_complexity_budget_roster.dag`)
  — COMPREP **wave-1** producers (`add`/`bind`/`branch`/`loop`). Subject reach is tied to COMPREP
  grammar coverage; more arrives with self-host breadth.
- Synthesis lens (`src/v2/lens/synthesis.dag`) is compiler-wide but **advisory** (existence
  lower-bounds don't yield a constructive patch — feasibility limit, not a wiring gap).

Blockers to "whole codebase": (a) a subject-producer for every fn (not name-keyed placeholders);
(b) the cost-lens loop **zero-absorption** that makes budgets toothless (`symbolic_max` floor fix);
(c) synthesis stays advisory by nature.
