# Effect grants over namespaces — hermetic/wet dissolved into (frame × verb × subtree)

Status: DRAFT for operator review (2026-07-16, session lively-heron-615). Origin: operator observation that "hermetic" conflates several concepts and that effect scoping should reuse **namespaces directly** rather than mint a parallel "universe" vocabulary (anti-fork ruling in the conversation; the term *universe* appears here only as the industry name for the idea being dissolved). No code lands from this doc; the FLAG-A interim (§6) is the only near-term consumer.

## 0. Displaced cost (§6 — the pain this removes)

- **`Hermetic | Wet` is a 2-valued enum carrying four concepts** (DESIGN's own recurring failure: state-space conflation). Casualties measured this week: a witness reading `/proc/self/status` had to declare `ReadsLiveTree` (never-skip) though it reads no tree — wrong label, only label available; the artifact-store witnesses write `/tmp/gunbc_*` scratch and the model cannot distinguish that from writing the repo tree or POSTing to a BMC; FLAG A (may a build run hermetically?) is undecidable *as phrased* because "inputs pinned AND outputs scoped" is inexpressible in one bit.
- **Proto-envelopes already forked**: the hand-rolled `workspace_root` containment gate (`cli_run.rs` — refuses paths outside the workspace root: literally a grant check, in Rust, unmodeled), `std.resources.ResourceHandle` (capability-ish carrier), `AuthScope` (cache_interface), `LiveTreeDisposition` (input-axis only, selection-eligibility only). Four partial representations of one concept (§3).
- **The A-lane and P3-native are about to pour concrete** into the old vocabulary (cross-worker store scoping; build admissibility). Separating now is cheaper than un-conflating later.

## 1. The four axes the current bit conflates

1. **Input closure** (replayability): does anything undeclared flow *in*? Decides cacheability/skip-soundness.
2. **Output reach** (interference): what may this run *affect*, and where? The "does not touch external systems" reading.
3. **Handler binding**: real transport vs recorded replay — a §3 *realization policy* per operation, today misused as the proxy for 1 and 2.
4. **Selection eligibility**: `LiveTreeDisposition` — axis 1 restricted to the repo tree, bolted on separately.

## 2. End shape (the one rule)

**An effect target is a position in a containment tree that already exists; permission is a grant of (verb × subtree) attached to a frame; admissibility is the namespace prefix relation.** No new tree-walk, no new scope noun:

- **Trees are cited, not minted** (§3): filesystem paths (the OS's own namespace, `extdeps/filesystem`), URIs (`extdeps/uri`), proc (`/proc/self/...`), service operation paths (the REST/Redfish hierarchies extdeps already models), and — the unifying case — **code names themselves** (the containment tree the namespace-resolution lane makes the single naming authority). One prefix-descent relation spans all of them: the same `⊑` the resolver walks, content-addressing hashes, and termination reads. Effects become the **fourth consumer** of the containment structure (namespace-resolution design's "one structure, N consumers", extended).
- **Grant** = `{ verb: Read | Write | Execute, root: NamespacePosition, binding: HandlerBinding }` where `HandlerBinding = RealTransport | Replay { fixture_store }` — axis 3 becomes per-grant policy, not a global mode. `Execute` is coarsened from `ExecuteEffect` via `verb_of_effect_shape` (invoke-nature shape; deploy-preflight sudo probes).
- **Envelope** = the grant set of a **frame** (`std.materialization_ladder.Frame` — already the scope/nesting authority). Child-frame envelopes are bounded by the parent's (lattice meet) — "at what times" is answered by frame lifetime; `ReplayedFrame`/`UnboundedSiblingsFrame` already model re-entry.
- **Dispatch checks the envelope fail-closed**: an effect whose target position is not under any grant root for its verb → typed, located, counted refusal (`EffectOutsideGrant { verb, target, frame }`). Never a silent widen; diagnostic modes read, no mode writes "keep going" (DESIGN §5).
- **Derived, not stored** (the projections that dissolve):
  - *replayable* (old "hermetic") ⇔ every Read grant is content-addressed (substrate closure) or `Replay`-bound;
  - *isolated* ⇔ every Write grant's root is a scratch subtree owned by the frame;
  - `LiveTreeDisposition` ⇔ "does any Read grant root intersect the repo tree" — computed, no longer stamped (retiring the machine-stamped-false class that let #6654's orphan wall get skipped);
  - `ExecutionMode = Hermetic | Wet | Record` ⇔ three common envelope presets, kept as *names for envelopes* during migration, deleted at the end.

**The containment law (operator refinement, 2026-07-16 — the basic cases that keep the model honest):** a write is **frame-contained** iff (target ⊑ a namespace the frame controls) ∧ (**target lifecycle ⊑ frame lifecycle**). The second conjunct is what the old "hermetic" intuition was actually about, and it is graded on the §5 construction/validation axis:

- `LifecycleByConstruction` — the target's persistence is *unwritable past the frame*: an ephemeral container's filesystem (teardown erases it, kernel-guaranteed), a network-namespaced loopback receiver that dies with the frame. Writes here are frame-contained in the strongest sense: the packet leaves the process, the file persists — and none of it can outlive the frame.
- `LifecycleByConvention` — a cleanup that can fail or be forgotten (`/tmp` scratch + deletion). Admissible, honestly weaker; the grade is part of the grant, never conflated.

**Acceptance cases the model must decide correctly (named up front so we don't mislead ourselves):**
1. *Loopback send to a controlled, netns-scoped receiver* → frame-contained (netns IS a namespace tree; receiver lifecycle ⊑ frame) — "hermetic" in today's terms despite being a real network send.
2. *File write inside an ephemeral container* → frame-contained by construction — "hermetic" despite real persistence during the frame.
3. *File write to `/tmp` scratch with trap cleanup* → frame-contained **by convention only** (the artifact-store witnesses' current honest grade).
4. *POST to a BMC / write into the repo tree* → not frame-contained under any grade; wet by any name.

**What this is not** (scope walls): quantities are not positions — memory/time budgets stay in the measure/CostAccount lane (an envelope bounds *reach*, a budget bounds *amount*); auth/identity is who-you-are, not what-you-may-touch (AuthScope converges only where it encodes reach); mutable keyed state stays a database (ladder's standing exclusion).

## 3. Convergence map (§2/§3 — DFS'd before minting; every element to its existing carrier)

| element | existing carrier | relationship |
|---|---|---|
| position / subtree / `⊑` | the containment tree + prefix descent (namespace-resolution design; `SymbolIndex` when it lands) | **reuse — the load-bearing anti-fork**: interim realization over path strings is the *filesystem's own cited tree*, not a second walk; convergence row: both become projections of one walk when the index lands |
| frame + nesting + re-entry | `std.materialization_ladder.Frame` | reuse |
| capability carrier | `std.resources.ResourceHandle` | converge (it already names resource_id + cap; gains a position + verb, loses stringly-ness) |
| the Rust path gate | `cli_run.rs` `workspace_root` containment refusal | first grant row to dissolve into the model (it is the proto-grant, hand-rolled) |
| handler binding | §3 shape/transport split; `Replay` = the recorded-fixture machinery (`claim_batch --hermetic/--record/--fixture-store`) | reuse — mocking becomes per-grant transport selection |
| refusal | typed diagnostic + counted-row discipline (P0 wall's `MissingField` pattern; ladder's `Refused` rows) | reuse the pattern |
| effect nature | `EffectShape`, ladder nature gates (`FreshEffect`/`WorldRead`) | grants reference shapes; verbs do NOT fork EffectShape |
| trees for world resources | `extdeps/filesystem`, `extdeps/uri`, service operation paths | cited trees; no new "resource kind" enum |

## 4. Phases

- **P-A (model, no behavior change):** grant/envelope types over `Frame`; the derivation fns (`replayable`, `isolated`, live-tree-intersection); witnesses over synthetic envelopes incl. REDs (target outside grant → typed refusal; child exceeding parent → refusal). `Hermetic`/`Wet` re-expressed as envelope presets, marked scaffold + dissolve-on.
- **P-B (first enforcement seam):** the `Filesystem` service dispatch checks Write targets against the frame envelope (the transport already carries `path` — the gate has a natural home); the `workspace_root` Rust gate re-derived from a grant row (its refusal becomes the modeled `EffectOutsideGrant`). The artifact-store witnesses' `/tmp/gunbc_*` roots become declared scratch grants — the first honest "writes own scratch, affects nothing else" rows.
- **P-C (derive the old axes):** `LiveTreeDisposition` computed from envelopes on witness entries (the stamp retires; the falsifier keeps enforcing); claim runners select handler bindings per grant instead of per-process mode.
- **P-D (FLAG A dissolves):** build admissibility = an envelope row (§6), no global ruling.

Each phase green-by-execution with REDs; under-scope is a counted frontier, never silent (P0 discipline).

## 5. Open questions (operator)

1. Verb set: start closed `{Read, Write}`? (`Execute`/`Create` as later rows when a displaced cost names them.)
2. Interim tree realization: path-string prefix over the OS/URI trees now, converging onto `SymbolIndex` when the naming lane's index lands — acceptable staging? (The convergence row is the anti-fork commitment.)
3. Does `AuthScope` converge here (where it encodes reach) or stay identity-side?

## 6. FLAG A interim (unblocks P3-native without cementing the old vocabulary)

A build is admissible in a replay-lane run when its envelope is: **Read ⊑ {substrate closure, pinned toolchain root (version in the artifact key), own workspace subtree}, Write ⊑ {own workspace subtree}, no network grant**. Recorded as the first envelope row (`build_workspace_grant`), scaffold-marked, dissolve-on = P-B enforcement seam. This is the operator-reviewable form of "yes, FLAG A" — a row, not a mode exception. (The workspace **Read** grant is the ownership-implies-read resolution settled by P-B — a build reads the emitted sources it was handed and the intermediates it produces; see below.)

**LANDED (PR-1, model + witnesses):** `build_workspace_grant.build_workspace_grant_envelope` is the **single** build-envelope authority over the landed `std.effect_grant` model; `v2.test.claim.build_workspace_grant_witness` proves it green-by-execution (network refused fail-closed by construction — no `UriTree` grant, so `admit_effect` has no arm to widen; live-repo-tree read refused while content-addressed-closure read of the same input is admitted; writes scoped to the workspace; workspace read admitted; refusal located). Its dissolution trigger stays **P-B** (let `emit_host_run_transport` run only on `Admitted`).

**LANDED (PR-2, the enforcement decision):** `build_transport_admission.build_transport_admissible` **consumes** that single authority (no forked envelope) and decides admissibility of a real `HostTransportDescriptor` — projecting each declared write (workspace files) and read (toolchain / produced programs) onto a `NamespacePosition` and folding `admit_effect`, returning the first located `EffectOutsideGrant` or `Admitted`; a workspace path that escapes (`..` / absolute) is refused. Green-by-execution in `v2.test.claim.build_transport_admission_witness`. This resolved the ownership-implies-read question PR-1 deferred: the workspace is **Read+Write** in the one authority.

**LANDED (PR-3, the enforcement wiring + two containment refinements):** the admission decision is now a **construction wall inside the compiler dispatch**, not a bypassable check. `v2.compiler.emit_host.run_host_process_with_cache` computes `build_transport_admissible(transport)` before any dispatch: `EffectOutsideGrant` → a located `Rejected` (`^emit_host_transport_outside_grant`) with **no** intrinsic call; `Admitted` → the existing build/run body (factored into `run_host_process_admitted`). Every existing green host-run witness (admitted transports) stays green by execution; the discriminating RED is `emit_host_transport_outside_grant_refuses_holds` — an escaping-workspace transport routed through `run_host_process` is refused *with that reason* (goes red if the gate is unwired or mis-armed). Two refinements fold in: containment is **segment-decomposed** (a `..` path segment escapes, `foo..bar` does not — review 39707), and an invocation's **args** are checked, not just its program (an escaping `WorkspacePath`/`ProducedArtifact` arg is refused — review 39684; exercised by the real python/go/ts/c descriptors). **Layering (§3 decision):** the gate must sit inside the compiler dispatch to be a construction wall covering all callers without inverting the layer DAG, so `build_workspace_grant` + `build_transport_admission` moved from `v2.workflow.*` to `v2.compiler.*` — they depend only on `std`, sit adjacent to their sole consumer (the emit-host mechanism), and their abstract roots are structural (concrete per-run roots remain a workflow concern when a production driver lands). **Remaining (P-B remainder, this lane):** three hand-rolled write refusals dissolved into `admit_effect` construction walls — `v2.extdeps.runtimes.v2_effect_io_host.effect_io_host_dispatch_write` (grant: `effect_io_host_grant`), `extdeps.realization.artifact_store_fs` (grant: `std.artifact_store_scratch_grant`), and the `cli_run.rs` `repo_relative_path` proto-grant (modeled authority: `gunbc.cli_run_repo_grant`; Rust HAND-RUST until Chunk F). Witnesses: `effect_io_host_grant_witness_test`, `artifact_store_scratch_grant_witness_test`, `cli_run_repo_grant_witness_test`. **Remaining (the actual hermetic-native run):** the seed intrinsic still refuses *all* host execution in hermetic mode (`is_hermetic()` blanket) — deliberately unchanged, because relaxing it to run hermetic builds requires the **network-isolation realization** (the model's "no network by construction" must be enforced by the realization: netns / `--offline` / sandbox, not just an absent grant), else it is a §5 fail-open on the network axis. That realization + the intrinsic relaxation is the next step; the admission decision it will consume is now landed and gating.

## 7. Ownership, de-fork sequencing, and decision log (silent-ibex-417, 2026-07-17)

**Ownership.** This doc and its implementation are owned by silent-ibex-417 as of 2026-07-17 (operator handoff: "you'll be owning that doc now/implementation"; earlier: "you can subsume that doc in your own PR as the work lands"). Originating draft is lively-heron-615's, currently also inside PR #6738. Handoff mechanics (Q4): this lane is now the doc's single home. #6738 still carries a copy; the originating session (`lively-heron-615`) is unreachable (archived), so the duplicate stays until #6738 resolves — the fork is a trivial new-file conflict (take this lane's version) at whichever merges second, not a content divergence.

**The concept being decomposed (operator working session, 2026-07-17).** "Effect" is currently carried by ≥5 partial, overlapping, forked vocabularies, each a flat enum that fuses several separable grounded axes:

- `std.effects.EffectShape` — CRUD verb (`Read`/`Upsert`/`Delete`/`Create`/`Append`) + `key_source` + *derived* idempotency, from HTTP method + path. **REST operation semantics.**
- `v2.std.effects.EffectShape` — idempotency-class partition (`IsIdempotent | IsBreaking`) + `Node` projection. **Substrate witness.**
- `std.temporal_effect.TemporalEffectPolicy` — inert third vocab (0 consumers; its `CreateIfAbsent` name-collides `std.effects.CreateCause`).
- `std.materialization_ladder` nature gates — `FreshEffect` / `WorldRead`.
- this doc's grant/envelope — verb × subtree × frame + handler binding. **Effect reach / permission.**

Plus the proto-envelopes §0 names (`ResourceHandle`, `AuthScope`, `LiveTreeDisposition`, the `workspace_root` Rust gate). A faithful `Effect` decomposes into: **nature** (read/write; CRUD is the fine grain) · **reach** (a position in a containment tree — this doc) · **key/target** (how the position is located) · **idempotency** (retry-safety — the `std.effects` derivation) · **handler binding** (real vs replay). The two `EffectShape` forks are anemic *cuts* of this record; grants is the reach layer standing **above one** `EffectShape`, not a replacement for it.

**The `std.effects` ↔ `v2.std.effects` de-fork is a PREREQUISITE, not a rival question.** §3 rules "grants reference shapes; verbs do NOT fork `EffectShape`" — grants stand on a *single* `EffectShape`, and two exist, so unifying them is the foundation grants sits on and runs first. It splits by axis:

- **Operation-semantics part** (`EffectShape` verb, `CreateCause`, and the three duplicate predicates `key_source_eq` / `create_effect_is_dedupable` / `create_double_init_collapsible`) → de-forks onto `dag/std/effects.dag` now. Authority settled: 69-vs-0 consumer census, `CompositeKey` live in `fleet_host_budget` / `host_identity_converge`, #6715 precedent (v2 dissolves onto dag). Grants-independent.
- **Key/target part** (`KeySource`) → its real home is this doc's containment tree (path-strings now → `SymbolIndex` later). A heavy rename-apart in the de-fork would be §2 redundancy — moved, not removed — so the de-fork does the *minimal* thing with `KeySource` and grants re-grounds it.

This is also what dissolves the **String model↔realization fork** that blocked a naive merge (`std.effects` = kernel `String`/`Value::Str`; the intern bridge needs `v2.std.text.String = FreeMonoid<Char>`): grounding key/target on the containment tree, staged, is where it resolves — never a crossing minted at the effects seam.

**Build steer (operator, 2026-07-17): model faithfully, stub what is not immediately consumed.** P-A models the whole decomposition with faithful shapes; unconsumed arms are honest stubs, each landing with a named dissolution trigger (DESIGN §6). Shape/concept fidelity is the bar — realization coverage is not.

**Sequence:** (a) de-fork the operation-semantics part of effects onto dag (prerequisite) → (b) **P-A** — grant/envelope model over `Frame`, no behavior change, faithful shapes + stubs → (c) **P-B..P-D** derive the old axes and dissolve the conflations.

### Decision log — operator rulings (filled as they land)

| # | question | recommendation | ruling |
|---|---|---|---|
| Q1 | Verb set — start closed at `{Read, Write}`? (`Execute`/`Create` as later rows only when a displaced cost names them.) | Yes | **AMENDED — Execute grounded via ExecuteEffect** (deploy-preflight displaced cost fired dissolve-on 2026-07-19; `verb_of_effect_shape(ExecuteEffect) = Execute`); `Create` still deferred |
| Q2 | Interim target realization — path-string prefix over the OS/URI trees now, converging onto `SymbolIndex` when the naming lane lands; the convergence row is the anti-fork commitment. | Yes, with the convergence row as a hard commitment | _pending_ |
| Q3 | `AuthScope` — converge into the grant model where it encodes reach, or stay identity-side? | Defer: identity-side for P-A; revisit when a reach-encoding consumer names the cost (avoid speculative convergence). | _pending_ |
| Q4 | Doc handoff from #6738 — take into this lane (drop from #6738), or land #6738 first and extend on main? | Take into this lane | **RESOLVED — take into this lane** (operator, earlier: "subsume that doc in your own PR"; "you'll be owning that doc"). #6738's copy is orphaned (session archived); trivial new-file conflict at merge. |

**Sibling thread (2026-07-21, session gentle-otter-138):** node/subtree visibility — private/public toggling over the containment tree — decomposes into two verbs riding this same grant algebra, not a parallel mechanism: `Reference` (compile-time edge-formation admission, inverted direction: root = admitted referrer subtree, not a target a frame may reach) and `Publish` (storage-realization/audience admission over a new cited `AudienceScopeTree`, absorbing `std.cache_interface.VisibilityScope`). The anti-fork search re-examined this doc's own convergence-map deferrals (`ResourceHandle`, `AuthScope`) and reconfirmed both as orthogonal peers, not merge targets. No changes to this doc's landed P-A/PR-1/PR-2/PR-3 model. → [node/subtree visibility grants design](docs/plans/node-subtree-visibility-grants.md)
