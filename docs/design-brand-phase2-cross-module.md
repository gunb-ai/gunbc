# Design: Cross-Module Identity at Scale — brand phase 2 (TODO A.2)

> **Status: DESIGN — completes the partially-designed phase 2.** Phase 1 (the `binding_id`
> side channel: ctrl#4579 + Amendment 4 #4587, impl gunbc#4581 — **out-of-clone**, cited per
> operator report) stamps declaration identity within a single resolved graph. Phase 2 is
> identity **across modules**: same-spelling declarations in different modules, imports,
> re-exports, and a multi-module compile of the compiler's own corpus (self-host breadth).
> Field/equality policy authority: `design-node-identity-channels.md` — this design lands
> through its table, rule 1 (#4581 defines the channel; phase 2 rides it).

## 1. Problem

Self-host means compiling `src/v2/**` as one multi-module corpus — hundreds of modules with
heavy same-spelling reuse (`Outcome`, `Node`, `Witness` declared once, imported everywhere;
plus genuine same-spelling *distinct* declarations across modules). Phase 1 makes identity a
stamped fact instead of a spelling within one graph. The phase-2 questions:

1. Do two same-spelled declarations in different modules get **distinct** identities by
   construction?
2. Do imports and re-exports **reference** identity rather than re-minting or re-deriving it
   from spelling at the module boundary (the boundary is where spelling-joins sneak back)?
3. Is identity **deterministic across the corpus** — independent of module admission order —
   so DB-8 and the self-host fixed point don't inherit order-dependence through ids?

## 2. What already exists (M9 DFS)

| Piece | Where | Role |
|---|---|---|
| Cross-file admission machinery: `Import`, `ImportVisibility`, `ResolutionSubject`, `Admission`, `AdmittedSubject`, `AdmissionState`, module-root validation | `src/v2/compiler/03_name_resolve.dag` (T-28-B) | **the seam**: admission is where cross-module references resolve — identity must ride through it as a fact |
| Single-file use→def resolution (canonical-spelling substitution today) | `03_resolve.dag` (K-1) | phase-1's stamping site; phase 2 extends what admission *carries*, not how K-1 works |
| `QualifiedName = FreeMonoid<Symbol>` | `std/qualified_name.dag` | the declared identifier of a module/declaration — the minting key's first half |
| `binding_id` channel: allocator + stamping seam + equality participation (**YES**) | #4581 (out-of-clone impl) + `design-node-identity-channels.md` | the carrier; phase 2 adds no second identity vocabulary |
| Allocator-determinism obligation | `design-node-identity-channels.md` rule 5 | phase 2 is where this obligation **bites** (§4.3) |
| Consumers: A3/A4 call-arg checks (#4533/#4554), the T-9 BindsTo rider, the termination call graph, COMPREP callee refs | landed/queued | everything that reads identity gets cross-module correctness for free if §4 holds |

## 3. Principles (short, because phase 1 did the hard part)

- **Minting is per-declaration, at the declaring module, exactly once.** Identity =
  minted at the declaration site keyed by (module `QualifiedName`, declaration). Two
  same-spelled declarations in different modules get distinct ids *by construction* —
  question 1 needs no checking pass, only the rule that **nothing else mints**.
- **Imports reference; re-exports preserve.** An import binds a local spelling to a foreign
  `binding_id` through `Admission`/`AdmittedSubject` — the admitted subject carries the id,
  so post-admission resolution never consults spelling across the boundary (the cross-module
  twin of phase 1's intra-graph rule). A re-export forwards the same id (no re-mint); an
  alias declared *as a new type* mints fresh — that distinction is exactly the brand rules,
  unchanged, applied at module boundaries.
- **No spelling-joins at boundaries, enforced.** The phase-2 failure mode is a consumer
  "temporarily" matching qualified-name text to connect modules. Same verdict as the
  termination audit gave intra-graph: refused; the admission carrier is the only path.

## 4. Design

### 4.1 Carry identity through admission
`AdmittedSubject` (or its successor in the T-28-B shape) gains the foreign declaration's
`binding_id` as a typed fact populated at admission time. Downstream (`04_infer`
call-arg checks, T-9 BindsTo facts, COMPREP callee refs) read the id off the admitted
subject — one carrier, every cross-module consumer.

### 4.2 Same-spelling distinctness is then a theorem, with one claim guarding it
Mint-once + reference-through-admission ⇒ distinct declarations have distinct ids
regardless of spelling. The corpus-scale claim (§5) exists to catch *regressions of the
rule* (someone minting at an import site), not to verify arithmetic.

### 4.3 Determinism across the corpus (the load-bearing decision — escalated)
A sequential graph-global allocator makes ids a function of **module admission order**;
order-dependence then leaks into everything ids touch (equality, hashes, emitted
artifacts), violating the channel authority's rule 5 and threatening DB-8/fixed-point.
Two compliant options:

- **(a) Canonicalized allocation:** keep the phase-1 allocator, but allocate in a declared
  canonical corpus order (sorted module `QualifiedName`, then declaration order). Cheap,
  keeps #4581 unchanged; the canonical order becomes one more declared fact.
- **(b) Content-derived ids:** id = stable hash of (module `QualifiedName`, declaration
  path) — order-independent by construction, no allocator state; but it changes #4581's
  mechanism and id equality semantics (collision posture must be stated).

**Recommendation: (a)** for phase 2 — it honors fewer-variants, leaves the building #4581
intact, and the canonical order is checkable. (b) remains the dissolution target if the
canonical-order fact ever proves fragile. **Operator ruling requested** (it touches the
in-flight #4581).

### 4.4 What phase 2 explicitly does not do
No cross-*compile* identity stability (a fresh compile may re-mint; consumers are
per-compile — same scoping as PROV's occurrence ids, Q-P1). No global registry file, no
identity persistence. If incremental compilation later needs stable ids, that is a new
consumer arriving with its own design.

## 5. Consumers and minimal slice (E-10)

- **Consumers (exist):** the A3 call-arg suite (its cross-module case is currently the
  PD-3-DOGFOOD gap — phase 2 is what makes un-skipping the compiler's own modules sound);
  the T-9 BindsTo rider (cross-module edges need foreign ids); self-host stage B
  (per-module artifact comparison needs deterministic ids in anything emitted).
- **Minimal slice:** two fixture modules each declaring same-spelled `type Token`, a third
  importing both under aliases — claims: **green** — distinct ids; cross-module call-arg
  check rejects passing one for the other (A3, across the boundary, by execution);
  **green** — a re-export carries the original id (witnessed equal); **red
  (discriminating)** — a deliberate spelling-join shim at admission accepts the brand-twin,
  proving the claim catches the regression class; **red** — two corpus orders produce
  identical ids under §4.3(a)'s canonical order (order-perturbation claim).

## 6. Open questions
- **Q-B2-1 — §4.3 ruling**: canonicalized allocator (rec) vs content-derived ids.
- **Q-B2-2 — visibility interaction**: does `ImportVisibility` affect id propagation
  (private declarations' ids crossing via re-export chains)? Recommend ids propagate only
  where admission admits — visibility stays the admission question it already is.
- **Q-B2-3 — phase-1 doc reconciliation**: ctrl#4579/#4587 predate this doc; their next
  amendment should cite it (and the channel authority) as phase-2's home.

## 7. Non-goals
Persistent/incremental identity, cross-repo identity, any change to phase-1 stamping
semantics, any second identity field (the channel table governs).
