# gunbc — Roadmap

`DESIGN.md` is the authority for *why*. This is the **shape of the work** — a scannable,
dependency-ordered checklist that keeps us honest over time. **Checkboxes are authoritative for
progress**; detail lives in the linked plan docs — don't restate it here (no dual representations).
A task's real state is its branch/PR + the carrier marks.

Legend: `[x]` done · `[ ]` todo · **indentation = depends on the item it sits under**.

## 0. Fail-closed lock-down LANE — BLOCKS expansion into products

Cache flakes, un-wired lenses, complexity violations = one problem: modeled, not **enforced fail-closed
by execution in CI**. Lock the machine down before selling pieces. The lane = *audit how deep the root
goes → fix → make the class unwritable.* → [audit + checklist](docs/plans/fail-closed-lockdown.md)

**Audits — how bad / how deep:**

- [x] lens/gate wiring audit — most analytical lenses are inert (authored, no discovered gate)
- [x] fail-open code audit — cache lossy-digest + under-keyed memos + `unwrap_or_default` infer
- [x] **model↔realization fork audit — ROOT CONFIRMED** ([plan](docs/plans/model-realization-fork.md)): one seam (coproduct *modeled* vs native `Value` *realized*, ~13 per-site bridges); two sub-roots — numeric tower (grounds cleanly) + `Value::Null` overload (needs *splitting*, the deeper root)
- [ ] coercion/equality fail-closure audit (`Bool` & `Optional`/`Null`-sentinel straddles after the `==` fix)
- [ ] inference fail-open audit (return-type / record-field remaining after #5293)
- [ ] cache-purity audit — enumerate every cache; each needs a warm==cold oracle

**Fixes — make fail-open unwritable:**

- [ ] cache flakes physically impossible
  - [ ] content-key on **raw bytes** (kill `from_utf8_lossy` digest, `resolved_graph_cache.rs:146`)
  - [ ] content-hash keys for `parse_table_memo` + `pure_call_memo` (not position/address)
  - [ ] **warm==cold purity-oracle witness per cache** (discovered, RED on divergence)
  - [ ] realizer-key lens: executed digest ⊇ declared inputs, else RED (= §7 P1.1)
- [ ] self-host purity enforced — wire `regen_stage0 --verify` into the floor (= §3 keystone)
- [ ] complexity/cost enforced — zero-absorption fix → change-set budget gate (= §6)
- [ ] cache-redundancy fail-closed — land §7 P3 redundancy cut **before expansion**
- [ ] ground the root ([model↔realization fork](docs/plans/model-realization-fork.md)): (1) numeric tower `Int=GroupCompletion<Nat>` → straddle guard becomes dead code; (2) **split `Value::Null`** (None/Absent/miss/Violates → own carriers) — the deeper root
- [ ] promote-or-delete every inert lens; de-vacuum thin gates (emit_host 4-fixtures, advisory rosters)

**Meta:**

- [ ] **meta-invariant gate:** every `lens/*.dag` has a discovered fail-closed discriminating witness, or is removed (closes the "authored ≠ enforced" loophole permanently)

## 1. Session dashboard on `.dag` (backend only)

- [ ] idea → PR pipeline

## 2. idea → idea compiler (stop anchoring on code)

De-anchor the compiler from CODE as the medium: a program is a canonical `Node` (the *idea*);
ingest / emit / eval across **many media** via one grammar read both directions (§2 N+M, not N×M).
→ [plan](docs/plans/idea-machine.md) · Two axes:

- [x] **medium axis** — `Medium<R>` + `DecodeFidelity` carrier; `LanguageModel` unified (13 forks dissolved); Source/DagSource/TargetSource → `Medium<String>`; `compile(Eval) → EvalResult{value: Medium<Node>}`
- [ ] **language axis** — 15+ targets wave-1; English emit proven (`english_emit_add_test`)
  - [ ] English vocabulary closure → **fail-closed** English ingest (today's catch-all `english_token_word` is fail-open — also a §0 item)
  - [ ] English ingest round-trip (only emit proven today)
- [ ] cross-media targets beyond syntax — JSON / react / diagram as **first-class media** (not stringified)
  - [ ] `Medium<A> ↔ Medium<B>` homomorphisms (Realization pattern over media)
- [ ] `FidelityDisposition` compose-up → medium-level `DecodeFidelity` at the decode boundary
- [ ] eval runtime generalization (wave-1 literal pins → `wave1_model_core` primitives)

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

## 4. HTML / React rendering (the "website" sellable piece)

- [ ] react/html rendering stands up (real page, not fixture)
- [ ] add to the demo alongside the TypeScript emit (website + language, dogfoodable)

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
