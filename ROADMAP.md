# gunbc — Roadmap

`DESIGN.md` is the authority for *why*. This is the **shape of the work** — a scannable,
dependency-ordered checklist that keeps us honest over time. **Checkboxes are authoritative for
progress**; detail lives in the linked plan docs — don't restate it here (no dual representations).
A task's real state is its branch/PR + the carrier marks.

Legend: `[x]` done · `[ ]` todo · **indentation = depends on the item it sits under**.

**Sections are in priority order, top = now.** Bands: **stability / correctness** (§0–§4, the focus for
the next few days — CI is in here: a flaky floor means no gate protects anything) → **expansion**
(§5–§7) → **shelved** (§8).

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

- [ ] **numeric-tower grounding** ([plan](docs/plans/model-realization-fork.md)) — `Int=GroupCompletion<Nat>` → the `==` straddle guard becomes dead code. *Start here* (highest value / lowest risk). **Authoritative home for this item** — §5's *de-fork integer-row* and the fork plan §3.1 are the same work seen from the self-host side; they point here, no second checkbox
- [ ] **cache trustworthy** (**= §2 F2/F3/P1** — that is the authoritative home; this is a pure pointer, progress tracked there) — the same key-from-`inputs_considered` construction, listed here only because it is the in-window stability driver. Ship the **warm==cold oracle now as a detective** — it stops the cache lying *today* while the from-inputs construction is built behind it (detective and constructive coexist in time). *(first instance: child adhoc-cc232dbc-1be)*
- [ ] **rust-gate coverage by cadence-decoupling** (shared with §1; *not* "run-all per-PR" — proven CPU-hours, the v1 SEED compiler costs ~tens of CPU-sec per trivial test) — per-PR = a **cost-bounded subset** (budget-derived, replacing the hand-picked 3-filter), full coverage = a **periodic `--ignored` lane**; completeness = **every test runs on ≥1 cadence** (fail-closed). #5427 lands the per-PR half now
- [ ] **promote-or-delete every inert lens** + de-vacuum thin gates (emit_host 4-fixtures, advisory rosters); whole-corpus the `discrimination` enforcer
- [ ] **realization-vocabulary containment guard** — target-AST construction (`extdeps.languages.bash.program`: `ShellStmt`/`serialize_bash`, + any future per-language AST sidecar) may be imported ONLY at the realization edge — the `extdeps/languages/**` models + the emit fold (`05_emit`/`06_translate`/`candidate_generation`); any other importer is a `RealizationVocabularyLeak`. A **construction wall** reusing `lens/layering_imports` forbidden-edge machinery (N+M, not a new lens mechanism), not a new optimizer. The current sidecar consumers (10 at authoring time; the live grep is the authority, the roster shrinks to 0) are a **named, shrinking exception roster** (a ratchet during migration); **dissolve-on** = the bash-sidecar arc migrates each to `emit(intent, Bash)` ⇒ roster empties ⇒ guard flips to a pure wall ⇒ `program.dag` deletable. This is what makes `shell(intent())` a realization-edge feature, never authored inside consumer code. Ties to §6 *emission = ingestion⁻¹*; full audit + architecture: [emission-ingestion-inverse.md](docs/plans/emission-ingestion-inverse.md)

**Fenced OUT of this window (flag-days / the fan-out — after stability):**

- [ ] **split `Value::Null`** (None/Absent/miss/Violates → own carriers) — ~131-site substrate change, the deeper root; needs its own runway (this is what actually *closes* the fail-open class — until then the numeric guards are dead but the class is open)
- [ ] **self-host purity gate** — *a §5 deliverable, not a §0-contained one* (avoids the §0↔§5 cycle: the edge runs **§5 self-host purity → §0 expansion-gate → products**, never §0 owning a §5 piece). Entangled with the Route-A green build (seed hand-maintained via `patch_*`, regen drift, gate closed #5325); self-host last mile, not stability plumbing. Listed here only to record that *shipping any product depends on it*
- [ ] **cross-tree import activation** (§5) — load-bearing, escalate before editing
- [ ] **`Disposition` carrier** ([plan](docs/plans/disposition-carrier.md)) — a new typed carrier + ratcheting self-dissolving lens; unambiguously a new concept (the fan-out wearing a stability badge). **Parked** until after the window
- [ ] complexity-budget whole-codebase (§3) · cache-redundancy completeness (§2 P3) — lens residue, after the construction lands

**Meta — lock down the reasoning (the DESIGN §7 recursion):**

- [ ] **inert-lens hygiene (executable backstop):** every `lens/*.dag` is wired (a discovered fail-closed witness) or **deleted** — an inert lens is a lie. This *runs* over the corpus
- [ ] **construction-justification rule (layered ON TOP, authoring-time):** before adding a lens, justify why the class can't be construction; convert what can; lens-justify only the residue. Does **not** supersede the executable backstop above (per [DESIGN §6](DESIGN.md))
- [ ] **confront the skipped modeling decisions** — the `🟡` comment corpus is the backlog; each resolves to construction or a justified `Terminal` ([Disposition plan](docs/plans/disposition-carrier.md), fan-out)
- [ ] **axiom + syllogism lens** (DESIGN open thread #1) — every design claim a consequence-chain back to an axiom, no orphan, no cycle (the §4 acyclicity test turned on this document). *Manual pass run 2026-06-21 (bright-eagle-46 review of #5424): found one orphan (testgen backlink), un-pointered duplicate checkboxes (cache-key, numeric-tower), a §0↔§5 lane cycle, and an undeclared §7→§6 edge — all fixed in this file. The lens stays `[ ]` until it runs **executably** over the doc; the manual pass is the discriminating witness it must reproduce*

## 1. CI under control (the correctness floor)

A green-but-broken or flaky floor means **no gate actually protects anything** — so CI reliability is
upstream of every §0 correctness claim. Getting CI under control *is* stability. (Compute fabric lives
here: it's the substrate CI runs on; selling it as an infra piece is downstream.)

- [x] privacy (compute fabric)
- [ ] **floor runs the right things** — close the coverage holes by **cadence-decoupling** (shared with §0; the 3-filter allowlist was COST selection, not arbitrary — the v1 SEED compiler costs ~tens of CPU-sec per trivial test, so run-all-per-PR is off the table):
  - [ ] per-PR cost-bounded subset + measured `#[ignore="expensive: Ns"]` + a completeness lens ("no un-run test without a written reason" — legit §6 residue: rustc `#[ignore]` is unstructurable) — **#5427** (fierce-hawk-540)
  - [ ] **nightly `--ignored` lane** (load-bearing CI-gen via `gunbc ci`/`ci_spec`) so expensive + the 58 currently-`#[ignore]`'d tests have a destination; upgrades completeness to "every reasoned-ignore has a lane". *Owned by §1 (quick-ant-298), sequenced after #5431 (per-test cost = measurement keystone applied to tests); dissolves as v2 self-host shrinks the seed. ⚠ scheduled-workflow gen is load-bearing — escalate before editing the CI-gen machinery*
- [ ] **floor runs reliably & affordably** — memory-aware scheduling (spawn_width is memory-blind → deterministic OOM as the corpus grows; `ResourceEnvelope.memory` modeled but unwired) and kill build flakes (sccache corruption ⇒ false-green: exit-0 with no artifact)
- [ ] repo model (internal repo) on compute fabric
- [ ] CI on compute fabric
- [ ] *(downstream / expansion)* compute fabric as a sellable infra piece

## 2. Minimal work — caching by realization (fail-closed)

Gate: uncached non-redundant work is an **ERROR**, not "slow". The cache-key-from-inputs construction
here is the §0 in-scope "cache trustworthy" item. → [plan](docs/plans/realization-measurement-loop.md)

- [x] F1 scheduler gives heavy nodes budgeted width (#5421)
- [ ] F2/F3 `resolved_graph` key **derived from** declared `inputs_considered` (single authority — construction, not a validating lens; #5423's spec-only lens was the false-green proof)
  - [ ] P1 honest keys by construction (key = fn of declared inputs · stable transform-id · census parity is the residue check)
    - [ ] P2 one door: `realize(subject)` as sole API (dissolves hand-rolled `ParseTable`)
      - [ ] P3 reach → minimal layer + fail-closed completeness gate + supplier provisioning ← **core ask**
- [ ] P4 economic tier (measured cost → `Materialization` by cost) — Phase-0 **instrument done** (model side was already floor-enrolled; step timing already emitted; only peak-RSS was missing, closed by #5431 emitting `VmHWM`+spawn_width). Remaining = Phase-1 *consumer* (measured→plan feedback + width-fold), which also unblocks §1-C memory-aware width
- [ ] P5 native `content(T) = content_hash(subgraph)` — gated on B2
- blockers: [ ] B1 #5295 generic-instantiation (gates cross-shard `Share`) · [ ] B2 v2 cross-tree content-hash / increment-4 (gates P5)

## 3. Complexity budget gate (stability — validation; the rewrite *construction* design is deferred to §5)

**Operator decision (2026-06-21): budget-gate validation is fine for the stability window; the
algorithmic-cost *rewrite* design comes after stability, homed with self-hosting (§5).** A budget gate is
*validation* (§5 — it concedes the suboptimal state is writable, flags after the fact) — accepted here as
the in-window tool; the construction replacement (rewrite to the cheaper equivalent) is **expansion**, not
stabilization, so it does not belong in this window (same fencing as Disposition / `Value::Null`-split in
§0). Detection itself is **total by construction** (cost.dag U2 — every program has an asymptotic class
via the kernel-level fold), so the gate's reach is a *subject-production* limit (fn-body reflection), not
a detection one.

- [x] complexity lens projection is total over the kernel (cost.dag U2); the *gate* runs a curated subject roster (COMPREP wave-1: add / bind / branch / loop)
- [ ] cost-lens zero-absorption fix (`symbolic_max` floor) — makes budgets non-toothless (**#5437**)
  - [ ] a subject-producer for every fn (not name-keyed placeholders) (#5437 helper; the *whole-corpus* gate needs fn-body reflection)
    - [ ] complexity budget gates the whole codebase (gated on fn-body reflection)
- [ ] synthesis stays advisory (feasibility limit, not a wiring gap; by Rice optimality is a *ratchet*, not a wall — DESIGN §5)

*The rewrite-catalog construction design (detection-vs-enforcement, the catalog, `Unknown`-as-anemic-atom
dissolution) is preserved in [docs/plans/algebraic-rewrite-optimization.md](docs/plans/algebraic-rewrite-optimization.md)
and relocated to §5 as a post-stability expansion lane.*

## 4. Testgen as the bug-class oracle (coverage by construction)

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
- [ ] **affected-set = the completeness half** ([same plan](docs/plans/testgen-oracle.md) §3) — model the full repo-process universe (incl. meta) under the lens so nothing is a blind spot; selection-as-CI-gate stays shelved (retired 0-min; v2 corpus is cheap). Shares testgen's reflection blocker (`node_query` now exists). Ties to the §0/§1 rust-gate-coverage hole
  - [ ] *anemia lens?* (operator-parked open thread, DESIGN §2 leaf-side decomposition) — flag `String` leaves that hide named structure. Almost certainly **advisory** (knowing a leaf *should* decompose needs the richer source — near the synthesis-feasibility limit), not a hard gate. Decide whether to elevate

## 5. Self-host v2 → delete `src/v1` (expansion)

Anchor (do not flip-flop): `.dag` = truth; **purely self-hosting** (v2 emits its own seed, no stage0
hand-edits); emit **Rust + TypeScript**; then shrink the seed to zero.
→ [plan](docs/plans/v2-self-hosting.md) · [de-fork audit](docs/plans/dsl-v2-defork-audit.md)

**Adjacent expansion lane — algorithmic-cost rewrite engine** (the §3 *construction* design, relocated
here per operator 2026-06-21: it is expansion, not stability, and IR-rewrite / canonicalization is most
natural once `.dag` is the self-hosted truth). **Post-stability.**
→ [plan](docs/plans/algebraic-rewrite-optimization.md)

- [ ] rewrite common suboptimal patterns to the cheaper equivalent (`O(n²)→O(n)`, `O(2ⁿ)→O(n)`, polynomial-degree peel by **structural-redundancy keying**, not degree) — construction on the cost axis; **bulletproof where it fires**, **published finite catalog** (silence ≠ optimal), four-witness DONE bar; *⚠ canonical-form-is-truth adds a pipeline normalization pass — load-bearing, escalate*
- [ ] `Unknown` dissolved over time as an **anemic atom** (decompress→map→reduce), reusing the `Disposition` carrier — never a false pass (`Unknown ⇒ Violates`, already holds)
- [ ] `O(n^x)→O(n log n)` substitution class as **per-idiom** rules (operator-flagged open: is there a cleaner shared framing?)

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

## 6. idea → idea compiler (expansion — stop anchoring on code)

De-anchor the compiler from CODE as the medium: a program is a canonical `Node` (the *idea*);
ingest / emit / eval across **many media** via one grammar read both directions (§2 N+M, not N×M).
→ [plan](docs/plans/idea-machine.md) · Two axes:

- [x] **medium axis** — `Medium<R>` + `DecodeFidelity` carrier; `LanguageModel` unified (13 forks dissolved); Source/DagSource/TargetSource → `Medium<String>`; `compile(Eval) → EvalResult{value: Medium<Node>}`
- [ ] **language axis** — 15+ targets wave-1; English emit proven (`english_emit_add_test`)
  - [ ] English vocabulary closure → **fail-closed** English ingest (today's catch-all `english_token_word` is fail-open — also a §0 item)
  - [ ] English ingest round-trip (only emit proven today)
- [ ] cross-media targets beyond syntax — JSON / react / diagram as **first-class media** (not stringified)
  - [ ] `Medium<A> ↔ Medium<B>` homomorphisms (Realization pattern over media)
- [ ] **emission = ingestion⁻¹ extended past syntax** — host-effect / orchestration intent as medium-agnostic `Node`s that emit per-target by **rows** (the row-driven inverse already realized for languages in `06_translate`): (a) **diagnostic-realization rows** — `Diagnostic{Severity}` → `{Bash: echo>&2, GitHubActions: ::error::, Rust: eprintln}`, dissolving the hand-rolled `render_log_annotation` forward-emitter; (b) **orchestration-as-intent** — a `Pipeline`/`Step`/`Run`/`Check` vocab so transports author intent and `emit(intent, Bash)` renders shell. Oracle = the round-trip law `emit ∘ ingest = id`, honest per-medium via `DecodeFidelity` (§4/§7). Enabler: the bash-sidecar arc (dissolves `program.dag`); **enforced-not-eroded by §0's containment guard**. Full audit + architecture: [emission-ingestion-inverse.md](docs/plans/emission-ingestion-inverse.md)
- [ ] `FidelityDisposition` compose-up → medium-level `DecodeFidelity` at the decode boundary
- [ ] eval runtime generalization (wave-1 literal pins → `wave1_model_core` primitives)

## 7. HTML / React rendering (expansion — the "website" sellable piece)

**Depends on §6** — react/html is a *first-class medium* (idea-machine.md §3 item 3 / §4 "Website product"),
so this lane sits downstream of §6's "cross-media targets beyond syntax" item, not just the §0 expansion gate.

- [ ] react/html rendering stands up (real page, not fixture)
- [ ] add to the demo alongside the TypeScript emit (website + language, dogfoodable)

## 8. Session dashboard on `.dag` (SHELVED)

Product/infra tooling — shelved during the stability window (no `.dag`-correctness leverage right now).
The anemia-lens angle that lived near here is **not** part of the dashboard; it's tracked as the
parked leaf-side-decomposition lens under §4.

- [ ] idea → PR pipeline *(deferred)*
