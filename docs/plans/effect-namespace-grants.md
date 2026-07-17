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
- **Grant** = `{ verb: Read | Write, root: NamespacePosition, binding: HandlerBinding }` where `HandlerBinding = RealTransport | Replay { fixture_store }` — axis 3 becomes per-grant policy, not a global mode.
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

A build is admissible in a replay-lane run when its envelope is: **Read ⊑ {substrate closure, pinned toolchain root (version in the artifact key)}, Write ⊑ {own workspace subtree}, no network grant**. Recorded as the first envelope row (`build_workspace_grant`), scaffold-marked, dissolve-on = P-B enforcement seam. This is the operator-reviewable form of "yes, FLAG A" — a row, not a mode exception.
