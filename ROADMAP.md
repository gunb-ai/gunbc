# gunbc — Roadmap

`DESIGN.md` is the authority for *why*. This is the **shape of the work** — a scannable,
dependency-ordered checklist that keeps us honest over time. **Checkboxes are authoritative for
progress**; detail lives in the linked plan docs — don't restate it here (no dual representations).
A task's real state is its branch/PR + the carrier marks.

Legend: `[x]` done · `[ ]` todo · **indentation = depends on the item it sits under**.

## 1. Session dashboard on `.dag` (backend only)

- [ ] idea → PR pipeline

## 2. idea → idea compiler (stop anchoring on code)

- [ ] scope

## 3. Self-host v2 → delete `src/v1`

Anchor (do not flip-flop): `.dag` = truth; **purely self-hosting** (v2 emits its own seed, no stage0
hand-edits); emit **Rust + TypeScript**; then shrink the seed to zero.
→ [plan](docs/plans/v2-self-hosting.md) · [de-fork audit](docs/plans/dsl-v2-defork-audit.md)

- [x] front-end (parse / resolve / infer) over the whole tree
- [x] emit whole tree `--target rust` (well-typed under CI gate)
- [ ] de-fork dsl ↔ v2 (one std authority, no historical forks)
  - [ ] turn on cross-tree import (wired but fail-closed today)
    - [ ] collapse clear duplicates (algebra, logic, nat, reducible, measure)
    - [ ] resolve same-name/different-job pairs (integer, effects, float, coercion, node, verification)
- [ ] emitted crate `cargo build`s green (Route-A last mile)
  - [ ] real fixed point: `content_hash` stage1==stage2 (Stage C; dissolve placeholder hashes, T-15/T-20)
    - [ ] wire `regen_stage0 --verify` lockstep gate into CI — enforces **no stage0 hand-edits** (was closed #5325) ← **keystone**
      - [ ] dissolve seed hand-patches (`patch_*` / `HAND_MAINTAINED_STAGE0_FILES`) so the emitter emits the whole seed
  - [ ] TypeScript to first-class (target-completeness beyond the `add` slice)
  - [ ] seed-honesty discharge (Diverse Double-Compiling — trust the seed once)
  - [ ] collapse `src/v1` → pinned reproducible v2-emitted seed; delete the 154k hand-written compiler logic (terminal, not a big-bang `rm`)

## 4. HTML / React rendering

- [ ] get it working

## 5. Compute fabric

- [x] privacy
- [ ] repo model (internal repo) on compute fabric
- [ ] CI on compute fabric

## 6. Complexity / synthesis lens over the whole codebase

- [x] complexity lens gates a curated roster (COMPREP wave-1: add / bind / branch / loop)
- [ ] cost-lens zero-absorption fix (`symbolic_max` floor) — makes budgets non-toothless
  - [ ] a subject-producer for every fn (not name-keyed placeholders)
    - [ ] complexity budget gates the whole codebase
- [ ] synthesis stays advisory (feasibility limit, not a wiring gap)

## 7. Minimal work — caching by realization (fail-closed)

Gate: uncached non-redundant work is an **ERROR**, not "slow". → [plan](docs/plans/realization-measurement-loop.md)

- [x] F1 scheduler gives heavy nodes budgeted width (#5421)
- [ ] F2/F3 key-completeness lens + `resolved_graph` keyed by construction (#5423, in review)
  - [ ] P1 honest keys verified by execution (realizer-key lens · stable transform-id · census parity)
    - [ ] P2 one door: `realize(subject)` as sole API (dissolves hand-rolled `ParseTable`)
      - [ ] P3 reach → minimal layer + fail-closed completeness gate + supplier provisioning ← **core ask**
- [ ] P4 economic tier (measured cost → `Materialization` by cost) — needs Phase-0 timing
- [ ] P5 native `content(T) = content_hash(subgraph)` — gated on B2
- blockers: [ ] B1 #5295 generic-instantiation (gates cross-shard `Share`) · [ ] B2 v2 cross-tree content-hash / increment-4 (gates P5)
