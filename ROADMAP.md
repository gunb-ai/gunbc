# gunbc — Roadmap

`DESIGN.md` is the authority for *why*. This is the **shape of the work** — a scannable,
dependency-ordered checklist that keeps us honest over time. **Checkboxes are authoritative for
progress**; detail lives in the linked plan docs — don't restate it here (no dual representations).
A task's real state is its branch/PR + the carrier marks.

Legend: `[x]` done · `[ ]` todo · **indentation = depends on the item it sits under**.

## 0. Fail-closed lock-down LANE — BLOCKS expansion into products

Cache flakes, un-wired lenses, complexity violations = one problem: modeled, not made **impossible to
write**. The fix is *correctness by construction, not validation* — now an axiom in [DESIGN §5](DESIGN.md)
(lenses are the residue mechanism, §6); roadmap items reference it, don't restate it.
→ [audit + checklist](docs/plans/fail-closed-lockdown.md)

**This window = a few days of STABILITY — reduce the fail-open surface, not "lock" it.** Honest scope:
numeric straddle gone, cache trustworthy, inert lenses promoted, rust gate widened. The deepest root —
the `Value::Null` overload (~131 sites) — stays **OPEN** until its own runway; dead straddle-guards are
*not* a closed class ([fork plan](docs/plans/model-realization-fork.md) §3).

**Audits — how deep (done):**

- [x] lens/gate wiring audit — most analytical lenses are inert (authored, no discovered gate)
- [x] fail-open code audit — cache lossy-digest + under-keyed memos + `unwrap_or_default` infer
- [x] **model↔realization fork audit — ROOT CONFIRMED** ([plan](docs/plans/model-realization-fork.md)): one seam (~13 per-site bridges); two sub-roots — numeric tower (grounds cleanly) + `Value::Null` overload (needs *splitting*, the deeper root)
- [ ] remaining: coercion/equality (`Bool` & `Null`-sentinel straddles) · inference fail-open (return-type after #5293) · cache-purity (every cache) · CI-coverage-completeness (rust gate runs 3 suites of **60** `src/v1/tests`, the rest rot — `ci_spec.dag:160`)

**In-scope this window (localized, stabilizing):**

- [ ] **numeric-tower grounding** ([plan](docs/plans/model-realization-fork.md)) — `Int=GroupCompletion<Nat>` → the `==` straddle guard becomes dead code. *Start here* (highest value / lowest risk)
- [ ] **cache trustworthy** — key derived from declared `inputs_considered` + raw-bytes key (kills the verified flake `resolved_graph_cache.rs:146`); spec'd in **§7** (F2/F3/P1). Ship the **warm==cold oracle now as a detective** — it stops the cache lying *today* while the from-inputs construction is built behind it (detective and constructive coexist in time). *(first instance: child adhoc-cc232dbc-1be)*
- [ ] **widen/retire the rust gate** — run the v1 test set or explicitly retire it (no test exists-but-doesn't-run)
- [ ] **promote-or-delete every inert lens** + de-vacuum thin gates (emit_host 4-fixtures, advisory rosters); whole-corpus the `discrimination` enforcer

**Fenced OUT of this window (flag-days / the fan-out — after stability):**

- [ ] **split `Value::Null`** (None/Absent/miss/Violates → own carriers) — ~131-site substrate change, the deeper root; needs its own runway (this is what actually *closes* the fail-open class — until then the numeric guards are dead but the class is open)
- [ ] **self-host purity gate** (§3) — entangled with the Route-A green build (seed hand-maintained via `patch_*`, regen drift, gate closed #5325); self-host last mile, not stability plumbing
- [ ] **cross-tree import activation** (§3) — load-bearing, escalate before editing
- [ ] **`Disposition` carrier** ([plan](docs/plans/disposition-carrier.md)) — a new typed carrier + ratcheting self-dissolving lens; unambiguously a new concept (the fan-out wearing a stability badge). **Parked** until after the window
- [ ] complexity-budget whole-codebase (§6) · cache-redundancy completeness (§7 P3) — lens residue, after the construction lands

**Meta — lock down the reasoning (the DESIGN §7 recursion):**

- [ ] **inert-lens hygiene (executable backstop):** every `lens/*.dag` is wired (a discovered fail-closed witness) or **deleted** — an inert lens is a lie. This *runs* over the corpus
- [ ] **construction-justification rule (layered ON TOP, authoring-time):** before adding a lens, justify why the class can't be construction; convert what can; lens-justify only the residue. Does **not** supersede the executable backstop above (per [DESIGN §6](DESIGN.md))
- [ ] **confront the skipped modeling decisions** — the `🟡` comment corpus is the backlog; each resolves to construction or a justified `Terminal` ([Disposition plan](docs/plans/disposition-carrier.md), fan-out)
- [ ] **axiom + syllogism lens** (DESIGN open thread #1) — every design claim a consequence-chain back to an axiom, no orphan, no cycle (the §4 acyclicity test turned on this document)

## 1. Testgen as the bug-class oracle (coverage by construction)

Prevent the **next class** of bug, not the last instance: generate witnesses from the declared structure
(the construction move applied to tests — one generator over structure beats N hand-witnesses).
→ [audit + method](docs/plans/testgen-oracle.md). Audit finding: testgen is itself a §0 lock-down
subject — its generated output is **not floor-discovered** (zero `test fn`, not `*_test.dag`), has **no
drift gate** (output can fork from generator), and is **mostly hand-anchored** (only AlgebraLaw derives
structurally).

- [ ] gate the existing generated output — floor-discover `generated/` (or a regen==committed drift gate); closes coverage-by-illusion + drift with no new logic
- [ ] make CoproductExhaustiveness **structural** — route through `node_query.coproduct_arm_keys` over *every* declared coproduct (not a hand-roster); RED = a removed arm
- [ ] add a **cross-representation-equality** category — generate the straddle witness per modeled-coproduct × native realization (the `==` fork is the live root; testgen should cover the *class*)
- [ ] **the oracle method (retro):** for each bug class, record — is there a category? structural? output gated? A "no" on a *structural* class is the work ([map](docs/plans/testgen-oracle.md) §2)
- [ ] **affected-set = the completeness half** ([same plan](docs/plans/testgen-oracle.md) §3) — model the full repo-process universe (incl. meta) under the lens so nothing is a blind spot; selection-as-CI-gate stays shelved (retired 0-min; v2 corpus is cheap). Shares testgen's reflection blocker (`node_query` now exists). Ties to the §0 rust-gate-coverage hole
  - [ ] *anemia lens?* (operator-parked open thread, DESIGN §2 leaf-side decomposition) — flag `String` leaves that hide named structure. Almost certainly **advisory** (knowing a leaf *should* decompose needs the richer source — near the synthesis-feasibility limit), not a hard gate. Decide whether to elevate

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
- [ ] F2/F3 `resolved_graph` key **derived from** declared `inputs_considered` (single authority — construction, not a validating lens; #5423's spec-only lens was the false-green proof)
  - [ ] P1 honest keys by construction (key = fn of declared inputs · stable transform-id · census parity is the residue check)
    - [ ] P2 one door: `realize(subject)` as sole API (dissolves hand-rolled `ParseTable`)
      - [ ] P3 reach → minimal layer + fail-closed completeness gate + supplier provisioning ← **core ask**
- [ ] P4 economic tier (measured cost → `Materialization` by cost) — needs Phase-0 timing
- [ ] P5 native `content(T) = content_hash(subgraph)` — gated on B2
- blockers: [ ] B1 #5295 generic-instantiation (gates cross-shard `Share`) · [ ] B2 v2 cross-tree content-hash / increment-4 (gates P5)

## 8. Session dashboard on `.dag` (SHELVED)

Product/infra tooling — shelved during the stability window (no `.dag`-correctness leverage right now).
The anemia-lens angle that lived near here is **not** part of the dashboard; it's tracked as the
parked leaf-side-decomposition lens under §1.

- [ ] idea → PR pipeline *(deferred)*
