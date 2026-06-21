# gunbc — Roadmap

`DESIGN.md` is the authority for *why*. This is the **shape of the work** — a scannable,
dependency-ordered checklist. **Checkboxes are authoritative for progress**; detail lives in the linked
plan docs — don't restate it here (no dual representations). A task's real state is its branch/PR.

Legend: `[x]` done · `[ ]` todo · **indentation = depends on the item it sits under**.

**Priority order, top = now.** Bands: **stability / correctness** (§0–§4) → **expansion** (§5–§7) →
**shelved** (§8).

## 0. Fail-closed lock-down LANE — BLOCKS expansion into products

Cache flakes, un-wired lenses, complexity violations = one problem: modeled, not made *impossible to
write*. Fix = correctness by construction, not validation ([DESIGN §5](DESIGN.md)).
→ [audit + checklist](docs/plans/fail-closed-lockdown.md)

This window = a few days of STABILITY — shrink the fail-open surface, don't "lock" it. The deepest root
(`Value::Null` overload, ~131 sites) stays OPEN until its own runway ([fork plan](docs/plans/model-realization-fork.md) §3).

**Audits (done):**

- [x] lens/gate wiring — most analytical lenses inert (authored, no discovered gate)
- [x] fail-open code — cache lossy-digest · under-keyed memos · `unwrap_or_default` infer
- [x] **model↔realization fork — ROOT CONFIRMED** — one seam (~13 bridges); sub-roots = numeric tower + `Value::Null` ([plan](docs/plans/model-realization-fork.md))
- [ ] remaining: coercion/equality straddles · inference fail-open · cache-purity · CI-coverage-completeness ([detail](docs/plans/fail-closed-lockdown.md))

**In-scope this window:**

- [x] **numeric-tower grounding** (#5428) — `Int=GroupCompletion<Nat>`; `==` straddle guard now dead-in-corpus ([plan](docs/plans/model-realization-fork.md))
- [ ] **cache trustworthy** — authoritative home is §2 F2/F3/P1; ship the warm==cold oracle as a detective now
- [ ] **rust-gate coverage** (shared §1) — opt-level=3 restores Pop-A to per-PR (#5456); run-all-unless-`#[ignore]`d (#5427) ([cause table](docs/plans/ci-selection-vs-scheduling.md))
- [ ] **promote-or-delete every inert lens** + de-vacuum thin gates *(silent-wren-739)*
- [x] **realization-vocabulary containment guard** (#5445/#5453) — target-AST importable only at the realization edge (fail-closed, shrinking-roster); dissolve-on: bash-sidecar arc empties the roster → pure wall → `program.dag` deletable ([plan](docs/plans/emission-ingestion-inverse.md))
- [ ] **stage0 clone-census inert + seed regressed** to 21540 (~1138 over) — resolve by clone-reduction / substrate-migration, NEVER a cap-bump; #5427 `#[ignore]` is the interim *(fierce-hawk-540 via quick-ant-298)*

**Fenced OUT (after stability):**

- [ ] **split `Value::Null`** (None/Absent/miss/Violates → own carriers) — ~131-site substrate change, the deeper root; own runway
- [ ] **self-host purity gate** — a §5 deliverable, not §0 (avoids the §0↔§5 cycle)
- [ ] **cross-tree import activation** (§5) — load-bearing, escalate before editing
- [ ] **`Disposition` carrier** ([plan](docs/plans/disposition-carrier.md)) — a new concept; parked
- [ ] complexity-budget whole-codebase (§3) · cache-redundancy completeness (§2 P3) — residue, after construction

**Meta — lock down the reasoning (§7 recursion):**

- [x] **inert-lens hygiene backstop** — every `lens/*.dag` wired or deleted; runs over the corpus (#5433)
- [ ] **reachability-completeness lens** — every declared node (code carrier · doc · lens) reachable from a run-root, rostered, or deleted; generalizes #5433 to carriers + docs ([plan](docs/plans/inert-layer-lens.md))
- [ ] **gate-hygiene: a floor-enrolled gate must be green-on-main at merge** — roster-completeness assertion promoted to should-land ([plan](docs/plans/emission-ingestion-inverse.md) §2) *(quick-ant-298)*
- [ ] **construction-justification rule** (authoring-time) — justify why a class can't be construction before adding a lens ([DESIGN §6](DESIGN.md)) *(silent-wren-739)*
- [ ] **expressibility frontier** — partition each modeling discipline into wall / lens-residue / undecidable-review *before* gating ([plan](docs/plans/expressibility-frontier.md))
- [ ] **confront the skipped modeling decisions** — the `🟡` comment backlog ([Disposition plan](docs/plans/disposition-carrier.md))
- [ ] **axiom + syllogism lens** (DESIGN open thread #1) — every claim chains back to an axiom, no orphan/cycle; stays `[ ]` until it runs executably over this doc

## 1. CI under control (the correctness floor)

A flaky or green-but-broken floor means no gate protects anything — so CI is upstream of every §0 claim.
(Compute fabric lives here: the substrate CI runs on; selling it as infra is downstream.)

- [x] privacy (compute fabric)
- [ ] **floor runs the right things** — cadence = two axes: SELECTION (by *what changed*) vs SCHEDULING (by cost); cost never drives selection ([plan](docs/plans/ci-selection-vs-scheduling.md))
  - [x] opt-level=3 restores Pop-A to per-PR — #5456 merged
  - [ ] per-PR = #5427 run-all sound baseline, shrunk to the affected set (#5427)
  - [ ] nightly = full-corpus selector-backstop + non-hermetic residue (#5447 stood down; ⚠ CI-gen load-bearing) *(quick-ant-298)*
- [ ] **floor runs reliably & affordably** — memory-aware scheduling (spawn_width is memory-blind → OOM as the corpus grows) + kill sccache false-greens
- [ ] **tree-scoped builtin registry** (fail-closed) — global seed registry leaks intrinsics into the substrate compile; instance fix #5452, class fix (partition) open *(quick-ant-298)*
- [ ] repo model (internal repo) on compute fabric
- [ ] **CI on compute fabric** — derive every host knob from one measured `ResourceEnvelope`; ends the crash-or-idle swing ([plan](docs/plans/compute-envelope-model.md))
- [ ] *(downstream)* compute fabric as a sellable infra piece

## 2. Minimal work — caching by realization (fail-closed)

Gate: uncached non-redundant work is an ERROR, not "slow". The cache-key-from-inputs construction is the
§0 "cache trustworthy" item. → [plan](docs/plans/realization-measurement-loop.md)

- [x] F1 scheduler gives heavy nodes budgeted width (#5421)
- [x] F2/F3 `resolved_graph` key derived from declared `inputs_considered` — construction, not a lens (#5425)
  - [ ] P1 honest keys by construction — warm==cold purity oracle (#5429)
    - [ ] P2 one door: `realize(subject)` sole API — kernel inhabits `cache_interface.dag` (#5446); ParseTable dissolution is downstream of the dsl→v2 de-fork (§5)
      - [ ] P3 **resolve-cache enable** — cuts ~18% of floor wall; purity proven (616/616); gated on #5429 ← **core ask**
- [ ] P4 economic tier (measured cost → `Materialization`) — instrument done (#5431); remaining = the consumer feedback + width-fold
- [ ] P5 native `content(T) = content_hash(subgraph)` — gated on B2
- blockers: [ ] B1 #5295 generic-instantiation (gates cross-shard `Share`) · [ ] B2 cross-tree content-hash (gates P5)

## 3. Complexity budget gate (stability — validation)

Operator decision (2026-06-21): budget-gate validation is the in-window tool; the algorithmic-cost
*rewrite construction* is expansion, relocated to §5. Detection is total by construction (cost.dag U2);
the gate's reach is a subject-production limit (fn-body reflection), not a detection one.

- [x] complexity lens total over the kernel (cost.dag U2); the gate runs a curated subject roster
- [x] cost-lens zero-absorption fix — budgets non-toothless (#5437)
  - [ ] a subject-producer for every fn (#5437 helper; whole-corpus needs fn-body reflection)
    - [ ] complexity budget gates the whole codebase (gated on fn-body reflection)
- [ ] synthesis stays advisory (by Rice, optimality is a ratchet not a wall — DESIGN §5)

→ rewrite-catalog construction design preserved + relocated to §5 ([plan](docs/plans/algebraic-rewrite-optimization.md))

## 4. Testgen as the bug-class oracle (coverage by construction)

Prevent the next class, not the last instance: generate witnesses from declared structure.
→ [audit + method](docs/plans/testgen-oracle.md)

- [x] gate the generated output — floor-discover `generated/` (or regen==committed drift gate) (#5434)
- [x] CoproductExhaustiveness made structural — over every declared coproduct, not a hand-roster (#5441)
- [x] cross-representation-equality category — straddle witness per coproduct × native realization (#5449)
- [x] **the oracle method (retro)** — bug-class→mechanism map (generator/lens/wall); testgen owns A + B-routing only, rest are lenses/walls ([map](docs/plans/testgen-oracle.md) §2)
- [x] affected-set = the completeness half — model the full repo-process universe (#5430)
  - [ ] *anemia lens?* (parked, DESIGN §2 leaf-side) — likely advisory, not a hard gate; decide whether to elevate

## 5. Self-host v2 → delete `src/v1` (expansion)

Anchor (do not flip-flop): `.dag` = truth; purely self-hosting (v2 emits its own seed, no stage0
hand-edits); emit Rust + TypeScript; then shrink the seed to zero.
→ [plan](docs/plans/v2-self-hosting.md) · [de-fork audit](docs/plans/dsl-v2-defork-audit.md)

Adjacent lane — algorithmic-cost rewrite engine (the §3 construction design; post-stability, natural once
`.dag` is the self-hosted truth). → [plan](docs/plans/algebraic-rewrite-optimization.md)

- [ ] rewrite suboptimal patterns to the cheaper equivalent (`O(n²)→O(n)` …) — published finite catalog, bulletproof where it fires
- [ ] `Unknown` dissolved over time as an anemic atom (reuse the `Disposition` carrier)
- [ ] `O(n^x)→O(n log n)` substitution as per-idiom rules (open: cleaner shared framing?)
- [x] front-end (parse / resolve / infer) over the whole tree
- [x] emit whole tree `--target rust` (well-typed under CI gate)
- [ ] de-fork dsl ↔ v2 (one std authority, no historical forks)
  - [ ] turn on cross-tree import (wired but fail-closed today) — Step 1 in flight *(nimble-koi-625)*
    - [ ] collapse clear duplicates (algebra, logic, nat, reducible, measure)
    - [ ] resolve same-name/different-job pairs (integer, effects, float, coercion, node, verification)
- [ ] emitted crate `cargo build`s green (Route-A last mile)
  - [ ] real fixed point: `content_hash` stage1==stage2 (dissolve placeholder hashes)
    - [ ] wire `regen_stage0 --verify` lockstep gate into CI — enforces no stage0 hand-edits ← **keystone**
      - [ ] dissolve seed hand-patches (`patch_*` / `HAND_MAINTAINED_STAGE0_FILES`)
  - [ ] TypeScript to first-class (beyond the `add` slice)
  - [ ] seed-honesty discharge (Diverse Double-Compiling)
  - [ ] collapse `src/v1` → pinned v2-emitted seed; delete the 154k hand-written lines (terminal, not a big-bang `rm`)

## 6. idea → idea compiler (expansion — stop anchoring on code)

A program is a canonical `Node` (the *idea*); ingest / emit / eval across many media via one grammar read
both directions (§2 N+M). → [plan](docs/plans/idea-machine.md)

- [x] **medium axis** — `Medium<R>` + `DecodeFidelity`; `LanguageModel` unified (13 forks dissolved); `compile(Eval) → EvalResult{value: Medium<Node>}`
- [ ] **language axis** — 15+ targets wave-1; English emit proven
  - [ ] English vocabulary closure → fail-closed English ingest (today's catch-all is fail-open; also §0)
  - [ ] English ingest round-trip (only emit proven today)
- [ ] **cross-media targets beyond syntax** — JSON / react / diagram as first-class media (not stringified)
  - [ ] `Medium<A> ↔ Medium<B>` homomorphisms
- [ ] **emission = ingestion⁻¹ extended past syntax** — diagnostic + orchestration intent emit per-target by rows ([plan](docs/plans/emission-ingestion-inverse.md))
- [ ] `FidelityDisposition` compose-up → medium-level `DecodeFidelity`
- [ ] eval runtime generalization (wave-1 literal pins → `wave1_model_core` primitives)
- [ ] **invert hand-maintained artifacts** — emit each (ROADMAP flagship · doc indexes) from its `.dag` authority + drift-gate it (the ci.yml pattern) ([plan](docs/plans/invert-hand-maintained.md))
  - [ ] **PR-to-checkbox status derivation** — checkbox derived from PR merge state (the §3 authority), drift = committed box ≠ derived; pure model + discriminating witness landed, host-fed gate edge next ([plan](docs/plans/invert-hand-maintained.md) §6 step 4)

## 7. HTML / React rendering (expansion — the "website" sellable piece)

Depends on §6 — react/html is a first-class medium (idea-machine.md §3/§4), downstream of §6's cross-media item.

- [ ] react/html rendering stands up (real page, not fixture)
- [ ] add to the demo alongside the TypeScript emit (website + language, dogfoodable)

## 8. Session dashboard on `.dag` (SHELVED)

Product/infra tooling — shelved during the stability window (no `.dag`-correctness leverage right now).

- [ ] idea → PR pipeline *(deferred)*
