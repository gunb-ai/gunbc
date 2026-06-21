# gunbc — Roadmap

`DESIGN.md` is the authority for *why*; this is *what's in flight*. A task's real state is its
branch/PR and the marks on the carrier (SCAFFOLD / dissolve-on / 🟡), not this file.

## 1. Session dashboard on `.dag`

Backend only — NOT the frontend.

- idea → PR pipeline

## 2. idea → idea compiler

Stop anchoring on code.

## 3. Self-hosting v2 (Rust + TypeScript)

**Anchor (decided 2026-06-21, do not flip-flop):** `.dag` is the truth; v2 compiles itself to a
bit-identical fixed point. Emit to **multiple realizations — Rust AND TypeScript** (both first-class
self-host targets, not one-or-the-other). Three languages total: **dag** (authority) · **rust** ·
**typescript**.

**Terminal goal: delete `src/v1`.** Once v2's emitted compiler builds green and reproduces itself,
**delete `src/v1/stage0`** (~125k lines of Rust seed + the `gunbc`/`claim_executor`/`regen_stage0`/
`claim_batch`/`discover_owned_data`/`yaml_check` bins) and any redundant v1 files. The seed shrinks to
zero — literally.

Today: front-end **done** (whole tree parses/resolves/infers). Open: the emitted whole-tree crate
doesn't `cargo build` green yet (Route-A last mile); the fixed-point gate is fail-closed (Stage C,
gated on candidate generation); TS proven only on the `add` slice.

Plan (verified state + tracks + the v1-deletion gating): [docs/plans/v2-self-hosting.md](docs/plans/v2-self-hosting.md)

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

## 7. Minimal work — caching by realization (fail-closed)

**End-state invariant (the acceptance gate):** for every pure transform `T` (resolve, parse, typecheck,
*and* derived analyses like complexity-synthesis / affected-set — ties to §6), `realize(T)` returns a
result content-addressed by `hash(inputs ⊕ content(T))`, materialized at the **minimal `Placement`
spanning T's consumers (its reach)**. A **non-redundant** transform (shared across a layer boundary, or
recompute-cost > cache-cost) with **no cache supplier at its reach layer** is a located, typed **ERROR**
that emits a *request to provision a supplier there*. No silent `Recompute`; no under-keyed hit.

Spine (dependency-ordered): **F1** scheduler width (#5421 merged) · **F2/F3** key-completeness lens +
`resolved_graph` keyed by construction (#5423) → **P1** honest keys verified by execution (realizer-key
lens; stable transform-id; census parity) → **P2** one door: `realize(subject)` as sole API
(dissolves hand-rolled `ParseTable`) → **P3 (core ask)** reach analysis → completeness gate fail-closed
(land the redundancy-only cut first; cost-based half waits on P4) + supplier provisioning → **P4**
economic tier (per-transform measured cost → `Materialization` by cost) → **P5** v2 dissolution
(`content(T) = content_hash(subgraph)` native). External blockers: **B1** #5295 generic-instantiation
(gates cross-shard `Share`); **B2** v2 cross-tree content-hash / increment-4 (gates P5, turns
approximation into native).

Inhabits existing carriers (`RealizedStep`/`Materialization`/`Placement`, `CacheLayerPlan`, `reach`,
`reconcile`), not re-coined. Detailed plan: [docs/plans/realization-measurement-loop.md](docs/plans/realization-measurement-loop.md)
(⚠️ owned by zesty-deer-479; quick-ant-298's dependency-ordered refinement above should be **reconciled
into that one doc**, not forked).
