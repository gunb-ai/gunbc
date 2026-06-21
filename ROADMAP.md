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

- [x] **numeric-tower grounding** (#5428) ([plan](docs/plans/model-realization-fork.md)) — `Int=GroupCompletion<Nat>` → the `==` straddle guard becomes dead code (now dead-in-corpus, kept as fail-closed backstop). *Start here* (highest value / lowest risk). **Authoritative home for this item** — §5's *de-fork integer-row* and the fork plan §3.1 are the same work seen from the self-host side; they point here, no second checkbox
- [ ] **cache trustworthy** (**= §2 F2/F3/P1** — that is the authoritative home; this is a pure pointer, progress tracked there) — the same key-from-`inputs_considered` construction, listed here only because it is the in-window stability driver. Ship the **warm==cold oracle now as a detective** — it stops the cache lying *today* while the from-inputs construction is built behind it (detective and constructive coexist in time). *(first instance: child adhoc-cc232dbc-1be)*
- [ ] **rust-gate coverage** (shared with §1) — the per-test "expense" was **debug-build amplification, not intrinsic seed cost** (root-caused 2026-06-21): **opt-level=3 on the `v1-compiler` test profile (#5456) restores the ~21 Pop-A pipeline tests to per-PR** (28–118s → ~0.4s), so per-PR is **run-all-unless-`#[ignore]`d** (#5427), not a cost-bounded subset; the `--ignored` lane shrinks to the ~16 Pop-B wet-captures only; completeness = **every test runs on ≥1 cadence** (fail-closed)
- [ ] **promote-or-delete every inert lens** + de-vacuum thin gates (emit_host 4-fixtures, advisory rosters); whole-corpus the `discrimination` enforcer
- [ ] **realization-vocabulary containment guard** — target-AST construction (`extdeps.languages.bash.program`: `ShellStmt`/`serialize_bash`, + any future per-language AST sidecar) may be imported ONLY at the realization edge — the `extdeps/languages/**` models + the emit fold (`05_emit`/`06_translate`/`candidate_generation`); any other importer is a `RealizationVocabularyLeak`. A **construction wall** reusing `lens/layering_imports` forbidden-edge machinery (N+M, not a new lens mechanism), not a new optimizer. The current sidecar consumers (10 at authoring time; the live grep is the authority, the roster shrinks to 0) are a **named, shrinking exception roster** (a ratchet during migration); **dissolve-on** = the bash-sidecar arc migrates each to `emit(intent, Bash)` ⇒ roster empties ⇒ guard flips to a pure wall ⇒ `program.dag` deletable. This is what makes `shell(intent())` a realization-edge feature, never authored inside consumer code. Ties to §6 *emission = ingestion⁻¹*; full audit + architecture: [emission-ingestion-inverse.md](docs/plans/emission-ingestion-inverse.md)
- [ ] **stage0 clone-census ratchet went inert AND the seed regressed** (surfaced by #5427's rust-gate widening) — `ownership_stage0_census` is a *hard* `assert!` (non-emit stage0 `.clone()` ≤ 20402) but the narrow rust gate never executed it per-PR, so the seed drifted to **21540 (~1138 over budget)** undetected: §6 coverage-by-illusion **on the rust side** (the #5433 `.dag` inert-lens backstop does not cover ungated rust tests) **+ a core-thesis regression** — Rust *grew*, against "the seed shrinks toward zero". Resolve by **substrate migration / genuine clone-reduction, NEVER a cap-bump** (project spirit forbids cementing; the ratchet is *downstream* of migration) — a §3/§7 strategic owner call, escalate-before-touch (load-bearing seed). #5427's honest `#[ignore=failing]` ("do NOT bump the cap — resolve by clone-reduction/substrate-migration") is the correct interim. *Surfaced 2026-06-21 (fierce-hawk-540 via quick-ant-298)*

**Fenced OUT of this window (flag-days / the fan-out — after stability):**

- [ ] **split `Value::Null`** (None/Absent/miss/Violates → own carriers) — ~131-site substrate change, the deeper root; needs its own runway (this is what actually *closes* the fail-open class — until then the numeric guards are dead but the class is open)
- [ ] **self-host purity gate** — *a §5 deliverable, not a §0-contained one* (avoids the §0↔§5 cycle: the edge runs **§5 self-host purity → §0 expansion-gate → products**, never §0 owning a §5 piece). Entangled with the Route-A green build (seed hand-maintained via `patch_*`, regen drift, gate closed #5325); self-host last mile, not stability plumbing. Listed here only to record that *shipping any product depends on it*
- [ ] **cross-tree import activation** (§5) — load-bearing, escalate before editing
- [ ] **`Disposition` carrier** ([plan](docs/plans/disposition-carrier.md)) — a new typed carrier + ratcheting self-dissolving lens; unambiguously a new concept (the fan-out wearing a stability badge). **Parked** until after the window
- [ ] complexity-budget whole-codebase (§3) · cache-redundancy completeness (§2 P3) — lens residue, after the construction lands

**Meta — lock down the reasoning (the DESIGN §7 recursion):**

- [x] **inert-lens hygiene (executable backstop):** every `lens/*.dag` is wired (a discovered fail-closed witness) or **deleted** — an inert lens is a lie. This *runs* over the corpus (#5433)
- [ ] **gate-hygiene: a floor-enrolled gate must be green-on-main at merge, or not enrolled yet** — #5445's `realization_vocab` lens merged while its `clean_tree` witness was red on main (the 10-vs-11 roster floor-skew), reding the whole fleet (#5453 is the one-file keystone fix). The **roster-completeness assertion** ([emission-ingestion-inverse.md](docs/plans/emission-ingestion-inverse.md) §2, was "optional follow-up") is **promoted to should-land**: it asserts the frozen roster covers every current importer at merge time — catching exactly this floor-skew *without* deriving the roster (deriving it makes the guard vacuous, leak always 0). *Process-miss postmortem 2026-06-21 (quick-ant-298)*
- [ ] **construction-justification rule (layered ON TOP, authoring-time):** before adding a lens, justify why the class can't be construction; convert what can; lens-justify only the residue. Does **not** supersede the executable backstop above (per [DESIGN §6](DESIGN.md))
- [ ] **confront the skipped modeling decisions** — the `🟡` comment corpus is the backlog; each resolves to construction or a justified `Terminal` ([Disposition plan](docs/plans/disposition-carrier.md), fan-out)
- [ ] **axiom + syllogism lens** (DESIGN open thread #1) — every design claim a consequence-chain back to an axiom, no orphan, no cycle (the §4 acyclicity test turned on this document). *Manual pass run 2026-06-21 (bright-eagle-46 review of #5424): found one orphan (testgen backlink), un-pointered duplicate checkboxes (cache-key, numeric-tower), a §0↔§5 lane cycle, and an undeclared §7→§6 edge — all fixed in this file. The lens stays `[ ]` until it runs **executably** over the doc; the manual pass is the discriminating witness it must reproduce*

## 1. CI under control (the correctness floor)

A green-but-broken or flaky floor means **no gate actually protects anything** — so CI reliability is
upstream of every §0 correctness claim. Getting CI under control *is* stability. (Compute fabric lives
here: it's the substrate CI runs on; selling it as an infra piece is downstream.)

- [x] privacy (compute fabric)
- [ ] **floor runs the right things** — close the coverage holes by **cadence-decoupling** (shared with §0). **Root-caused 2026-06-21 (proud-deer-709 cause-table, [expensive-test-cause-table.md](docs/plans/expensive-test-cause-table.md)): the "expensive" tests were NOT intrinsic seed cost — they were DEBUG-BUILD AMPLIFICATION** (the per-PR rust gate ran `cargo test` in *debug*, so the unoptimized v1-compiler seed ran 100–800× slower; the release CLI does the same whole-tree index + resolve + typecheck in ~0.1s). Two populations: **Pop-A** (~21 in-process pipeline tests, 28–118s in debug) and **Pop-B** (~16 subprocess/network wet-captures):
  - [x] **opt-level=3 on the `v1-compiler` test profile restores Pop-A to per-PR** (**#5456**, warm-ram-537 — *MERGED 2026-06-21, squash `a2301d3317`; `[profile.test.package.v1-compiler] opt-level = 3` confirmed on main — note the squash title is mislabeled "Pop-B", content verified correct*): pipeline tests collapse 28–118s → ~0.4s each (377 in 151s); the one-time optimized build amortizes via sccache (run-2 0.09s). The coverage hole closes **directly** — Pop-A does **not** need cadence-decoupling. *"Most expensive = a defect, fix it don't schedule it away."*
  - [ ] per-PR **run-all-unless-`#[ignore]`d-with-a-written-reason** + a completeness lens (legit §6 residue: rustc `#[ignore]` is unstructurable) — **#5427** (fierce-hawk-540)
  - [ ] **nightly `--ignored` lane** (load-bearing CI-gen via `gunbc ci`/`ci_spec`) — **reduced scope post-opt-level: schedule the ~16 Pop-B subprocess/network wet-captures only; NO expensive-category selector** (Pop-A is per-PR). Pop-B is itself mostly (a)/(b) defects warm-ram is draining (rebuild-per-test → consume the prebuilt binary), so the lane shrinks to a small irreducible residue (live-network + genuinely-large input). *Owned by §1 (quick-ant-298); dissolves further as v2 self-host shrinks the seed. ⚠ scheduled-workflow gen is load-bearing — escalate before editing the CI-gen machinery*
- [ ] **floor runs reliably & affordably** — memory-aware scheduling (spawn_width is memory-blind → deterministic OOM as the corpus grows; `ResourceEnvelope.memory` modeled but unwired) and kill build flakes (sccache corruption ⇒ false-green: exit-0 with no artifact)
- [ ] **tree-scoped builtin availability (registry partition)** (fail-closed) — the seed `builtin_function_registry` (76 names, `src/v1/04_method.dag`, a marked bridge scaffold) is **global, not scoped to the compiled tree**, so v1-seed intrinsics leak into the dsl-substrate compile and *resolve without a real `.dag` def* (a §5 fail-open, surfaced by `utf8_decode_bytes` in `secret_manager.dag` — the gate is green despite no genuine definition). Fix by construction: the substrate may use only builtins with real `.dag` defs; seed-only names admitted only when entry-root = the v1 seed — this advances the registry's *own* sanctioned dissolve-on ("deleted when builtins are actual `.dag` definitions"). Load-bearing seed + a likely red wave → **measure-first** (leaked-name count: how many of the 76 the substrate relies on), expose-then-triage, escalate before the enforce-flip. *Owned by §1 (quick-ant-298); instance fix (real `utf8_decode_bytes` std fn in `std/encoding` **+** removal of the registry bridge entry & its seed mirror) = **#5452** (verified sound: symbol now resolves via the real `.dag` def, execution is the fail-closed `v1_rt::utf8_decode_bytes` intercept; fleet-gated). Class fix (tree-scoped partition) still open.*
- [ ] repo model (internal repo) on compute fabric
- [ ] CI on compute fabric
- [ ] *(downstream / expansion)* compute fabric as a sellable infra piece

## 2. Minimal work — caching by realization (fail-closed)

Gate: uncached non-redundant work is an **ERROR**, not "slow". The cache-key-from-inputs construction
here is the §0 in-scope "cache trustworthy" item. → [plan](docs/plans/realization-measurement-loop.md)

- [x] F1 scheduler gives heavy nodes budgeted width (#5421)
- [x] F2/F3 `resolved_graph` key **derived from** declared `inputs_considered` (single authority — construction, not a validating lens; #5423's spec-only lens was the false-green proof; landed #5425)
  - [ ] P1 honest keys by construction (key = fn of declared inputs · stable transform-id · census parity is the residue check) — **warm==cold purity oracle = #5429** (stern-otter; green/decoupled, the live falsifier for a non-lying cache)
    - [ ] P2 one door: `realize(subject)` as sole API. **Phase-1 realize KERNEL inhabits the previously-dead `cache_interface.dag` = #5446** (green/decoupled; kills the §3 Rust-fork direction). **The actual hand-rolled-`ParseTable` dissolution is a DOWNSTREAM CONSUMER of the dsl→v2 de-fork (§5), not independent:** keen-otter found the v2-local rewire is *cosmetic* — the v1 host (`v1_interpreter.rs`) is the sole live parse-memo authority and already content-addresses; the `.dag table.entries` map is vestigial (`parse_table_insert` discarded, `02_parse.dag:835`), so a .dag-only rewire feeds a dead map (§5 spec-without-execution). Only the content-key MODEL (#5455) + the #5446 kernel land pre-de-fork; dissolution onto the kernel needs the `v2.std.cache_interface` handler binding
      - [ ] P3 **resolve-cache enable = GO (cost-justified, not correctness-only)**: warming the floor resolve cache cuts **~18% of CI floor wall** (231s; resolve = 68.4% of discovery compute), purity **proven** (616/616 warm==cold verdict-identical, byte-identity by construction via #5425). Lands as enable `GUNBC_RESOLVED_GRAPH_CACHE_DIR` behind a **continuous** warm==cold shadow audit (the live falsifier, not a one-shot), **gated on #5429 merge**. *Distinct from Pop-A — the cache does NOT drain Pop-A (in-process cache-blind path); §2's lever is the CLI/floor resolve.* ← **core ask**
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
- [x] cost-lens zero-absorption fix (`symbolic_max` floor) — makes budgets non-toothless (**#5437**)
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

- [x] gate the existing generated output — floor-discover `generated/` (or a regen==committed drift gate); closes coverage-by-illusion + drift with no new logic (#5434)
- [ ] make CoproductExhaustiveness **structural** — route through `node_query.coproduct_arm_keys` over *every* declared coproduct (not a hand-roster); RED = a removed arm
- [ ] add a **cross-representation-equality** category — generate the straddle witness per modeled-coproduct × native realization (the `==` fork is the live root; testgen should cover the *class*)
- [ ] **the oracle method (retro):** for each bug class, record — is there a category? structural? output gated? A "no" on a *structural* class is the work ([map](docs/plans/testgen-oracle.md) §2)
- [x] **affected-set = the completeness half** (#5430) ([same plan](docs/plans/testgen-oracle.md) §3) — model the full repo-process universe (incl. meta) under the lens so nothing is a blind spot; selection-as-CI-gate stays shelved (retired 0-min; v2 corpus is cheap). Shares testgen's reflection blocker (`node_query` now exists). Ties to the §0/§1 rust-gate-coverage hole
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
