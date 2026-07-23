# C linkage-unit realization — the 2nd realization of the shared `CompilationUnit` shape

**Status:** design, built against the interface locked 2026-07-16 (`docs/plans/emitted-crate-partition-design.md` §2/§13). The shared home (`CompilationUnit` / `CompilationUnitId` / `partition_fold` / R4 validity interface / `Provenance`) is authored in parallel by the sibling lane (work-item `adhoc-85078326-fdb`); this doc does not define it, and implementation is gated on it landing + main-green. This doc's purpose is the design **and** the yes/no verdict on whether the locked shape genuinely expresses C's axes without contortion — the reason a 2nd, non-Rust realization was staffed at all.
**Relates to:** `docs/plans/emitted-crate-partition-design.md` (the Rust realization, sibling to this one, same shared home), `#6650` (C Phase 0 — cpp-model-through-`cc` offline witness, the `cc` host transport this doc's R3 build extends), DESIGN §2 Realization ("one concept, every scale/breadth" — Rust crate = C translation unit = Go package = .NET assembly are §3 nicknames for one concept), §3 single-authority / import-arrow, §5 fail-closed, §7 self-host.

---

## 1. Why C is the 2nd realization (the point of this lane)

A shape factored from one realization (Rust) risks cementing that realization's accidents as if they were universal — the §7 byte-fixpoint trap applied one level up, to *modeling* rather than emission. The shared `CompilationUnit` home is being authored with two axis-sets present at once (Rust + C) specifically so it cannot silently bake in a Rust-only assumption.

The sharpest such assumption is **acyclicity**. Rust crates (and Go packages) form a mandatory-acyclic linkage graph — the R4 in the Rust design doc refuses a cyclic partition outright. C is the opposite: translation units may **mutually `#include` and mutually reference each other's symbols** and still link cleanly, because C resolves symbol references at link time, not at a per-unit compile-graph level. If the *shared* shape (not just the Rust realization) had smuggled in "linkage units form a DAG," C could not express its own valid programs against it. So the test this doc runs is concrete: **take the same cyclic module partition that Rust R4 must refuse, and show C R4 must accept it** — against the *same* `CompilationUnit` type, with no shape edit.

## 2. The locked shared interface (recap — owned by the home, not redefined here)

- `CompilationUnit { members, interface (R5-derived), deps (R5-derived), artifact (opaque agnostic id) }`
- `CompilationUnitId` — agnostic.
- `partition_fold(module_dag, Map<ModuleId, CompilationUnitId>) -> List<CompilationUnit>` — the shared kernel; sole producer of `interface`/`deps` from boundary-crossing module edges.
- Per-target **R4 validity-predicate interface** — realized once per target.
- `Provenance = HandExplicit | PolicyDerived { policy: DeclarationRef }`.

Everything below is this doc's realization of R3 (linkage spelling) and R4 (C's validity predicate) against that interface — unchanged.

## 3. C R3 — linkage spelling

### 3.1 The `.h`/`.c` split (a genuine C axis Rust does not have)

Rust has no header/impl separation — a crate's public surface and its implementation are the same source, split by `pub`. C's separate-compilation model *requires* the split: a `.c` file is compiled alone, so anything a dependent unit needs to see (function prototypes, type definitions used by value, `extern` data declarations, macros) must exist as text the dependent's preprocessor includes — that text is the `.h`.

This maps directly onto the *existing* derived/input split in the shared shape, with no new field:

- `CompilationUnit.members` (input: the module-DAG nodes assigned to this unit) → renders to the unit's **`.c`** — the implementation.
- `CompilationUnit.interface` (R5-derived: the exported surface other units may reference) → renders to the unit's **`.h`** — declarations only, never definitions of non-`inline`/non-`static` functions (a `.h` that leaked a definition would violate C's one-definition rule the moment two `.c` files included it).

So the split is **not a new axis on the shared type** — it is what C R3 *does* with the two fields the shape already carries. This is the confirming case for the shape, not a gap: `interface` was specified as "exported surface," which is exactly a C header's job, independent of what target renders it.

One real requirement this surfaces for the home author: `interface`'s carried content must be rich enough to hold a **full type definition**, not just a name/signature — C requires the complete definition of any type used **by value** across a unit boundary (a struct passed or returned by value, not by pointer) to be visible at the use site, or the dependent unit cannot even parse the call. This is not a shape change (the field is already an opaque `Node`/carrier, not a flat symbol-name list), just a call-out: `partition_fold`'s derivation of `interface` must include full-definition content for by-value-crossing types, not merely a declared name, or the C realization has nothing correct to render into the header. Flagged to the home author; not a blocking finding (see §6).

### 3.2 Cross-unit reference

A boundary-crossing edge from `partition_fold`'s `deps` computation becomes, in C:

1. `#include "<dep-unit>.h"` in the depending unit's `.c` (and, if the dependency is itself part of the *interface* being re-exposed, in the depending unit's own `.h`).
2. A forward declaration (`struct Foo;`) instead of a full `#include`, when only a pointer/reference to an as-yet-incomplete type is needed — the cheaper option C offers that Rust's module system has no analog for (Rust re-exports are the same cost as the real item).

`deps` (the `CompilationUnitId` set) tells C R3 *which* headers to include; it does not need to know *whether* a full include or a forward declaration suffices — that is decided per-crossing from the *kind* of use (by-value vs by-pointer), which is exactly the "genuinely-invalid" boundary R4 also cares about (§4).

### 3.3 Translation-unit identity = include-closure

A C `.c` file plus everything it transitively `#include`s **is** the translation unit the compiler actually sees — headers have no separate compiled identity. So for the C realization, `CompilationUnit` identity at R3 time is the `.c` file; the **preprocessor include-closure** (this `.c`'s header plus every header transitively `#include`d, including other units' headers reached via `deps`) is a *derived, checkable fact* about that unit, not a new concept — it is the C-real instantiation of "what does this unit actually depend on to compile," useful for incremental-rebuild tracking exactly the way Rust's crate-dependency edge is. No shape addition: it is `deps`, transitively closed, spelled out as `#include` lines.

### 3.4 Artifact spelling

`CompilationUnit.artifact` (the opaque agnostic id) realizes, for C, as: the unit's `.o` object file, produced by `cc -c <unit>.c -o <unit>.o`, and the **link step** that combines every unit's `.o` (plus any unit whose `deps` are satisfied only at link time, e.g. a forward-declared function called across a link-cyclic pair — see §4) into the final binary via `cc <unit1>.o <unit2>.o ... -o <program>`. This composes with the existing `cc` host transport already registered for the cpp/C target (`#6650`) — no new transport, just a multi-file argv instead of single-file.

## 4. C R4 — the validity predicate (the key contrast with Rust)

**Permissive on cyclicity, by design:** a partition whose unit-condensation is cyclic — unit A's `.c` calls a function only declared (not defined) by unit B, and B's `.c` symmetrically calls one only declared by A — is **valid** for C. The linker resolves both references once every `.o` is present; nothing about C's separate-compilation model requires the *unit dependency graph* to be acyclic. This is the literal negation of Rust R4's refusal, realized against the identical shared `CompilationUnit` shape with zero edits to the shape itself — R4 is exactly the seam the locked interface designated as per-target for this reason.

**What genuinely is invalid for C** (fail-closed, §5 — refuse, never widen) is narrower and different in kind from Rust's constraint:

- **Cyclic complete-type containment.** If type `Foo` (defined in unit A) contains type `Bar` **by value** (not by pointer) and `Bar` (defined in unit B) contains `Foo` by value, no forward declaration can break the cycle — C requires a complete type to lay out a by-value member, and the two sizes are mutually dependent. This is a real refusal, independent of unit linkage: it exists whether or not A and B are partitioned into the same or different units, but partitioning them into *different* units is exactly where it becomes externally visible (same-unit, the compiler would already reject the single translation unit). The predicate: for the sub-relation of `deps` restricted to *by-value* type crossings, that sub-relation must be acyclic; a cycle in it is a **typed, located refusal** naming the two types and the crossing edge, not a silent "just include both headers and hope."
- **Split-definition (ODR) violation.** A non-`static`, non-`inline` symbol's *definition* must live in exactly one `.c`; if `partition_fold`'s member assignment ever placed the same definition-bearing module node into two units (a fold defect, not a legitimate partition), C R4 refuses — this is the C-real instance of the shared shape's own invariant that `members` partitions the module set (each module in exactly one unit), so it is close to a sanity check on `partition_fold`'s output rather than a fresh C rule, but it is stated here because C's linker turns a violation into a *duplicate-symbol link error*, which is the discriminating signal (§6.2).

Both refusals are **typed and located** (name the module/type pair and the crossing edge), never a widened "recompile everything" or "just merge the units" fallback (§5 — no absorbing fallback).

## 5. Acceptance — grounded on determinism, not a new concept

Per the operator reframe already applied to the Rust realization: the unit partition is a **non-semantic perturbation axis** (`v2.std.determinism` / `std.perturbation`). Behavior must be invariant under *how* the module graph is cut into `CompilationUnit`s; a behavior that depends on the cut is a determinism **leak**, not a partition-specific concept. No second acceptance mechanism is invented for C.

**Discriminating witnesses** (green-by-execution, §5/§7 — proven by running `cc`, not by a structural check):

1. A fixed C-target module DAG, partitioned **1-unit** (everything in one `.c`) — compiles, runs, behaviorally correct.
2. The same module DAG partitioned **N-unit acyclically** — compiles (one `.o` per unit, linked), runs, behaviorally equivalent to (1).
3. The same module DAG partitioned so the unit-condensation is **cyclic** (the case Rust R4 would refuse) — compiles (per-unit `.o`s, forward-declared cross-calls resolved at link time), runs, behaviorally equivalent to (1) and (2). **This is the load-bearing witness**: the cyclic partition *compiling and matching* is the proof the C axis is real and the shared shape did not silently exclude it.
4. A partition engineered to hit the genuinely-invalid case (§4 — a cyclic by-value type containment split across units) — **refuses**, typed and located, before `cc` is ever invoked (a refusal at fold/R4-check time, not a `cc` compile error surfacing the same defect two layers down — though a `cc` compile error on this input would itself falsify the refusal if it fired instead, so the receipt should show the typed refusal firing *and* the naive unrefused version failing at `cc` as the negative control).

Precedent for the execution harness: `#6650`'s cpp-model-through-`cc` offline witness — extended from a single fixed `add` source to the ≥2-partition family above, same `cc` transport, multi-file argv per §3.4.

## 6. The yes/no verdict

**Does the locked `CompilationUnit` shape express C's `.h`/`.c` split and link-cyclic axis without contortion? Yes.**

- The `.h`/`.c` split needs no shape change: it is C R3's rendering of the existing `members`(input)/`interface`(derived-output) fields, which is precisely what those fields were specified to mean (§3.1).
- The link-cyclic axis needs no shape change: it is a C R4 predicate that accepts what Rust R4's predicate refuses, over the identical `CompilationUnit` values — R4 was designated per-target for exactly this reason, and the contrast is clean, not forced.
- `artifact` as an opaque id needs no shape change: it realizes as `.o` + link-step for C exactly as it realizes as crate-name + `rustc` invocation for Rust (§3.4).

**One non-blocking call-out to the home author (not a shape gap, a richness requirement):** `partition_fold`'s derivation of `interface` must be able to carry a **full type definition**, not only a declared name/signature, for any type crossing a unit boundary by value (§3.1). If the home's current draft models `interface` as a flat set of symbol names, C R3 has nothing correct to lower into a header for by-value crossings and would have to reconstruct the definition out-of-band — that *would* be a contortion. Since `interface`'s carrier was specified as an opaque `Node` (not a name list), this is expected to already be expressible; flagging so the home author can confirm the derivation actually walks to full definitions where the crossing is by-value, not just symbol identity. No escalation filed — this is a confirmable detail, not a discovered shape break.

## 7. Staging / gating

1. **Design (this doc) — no in-tree shared home required.** Deliverable now.
2. **Implementation — gated on:**
   - the shared home (`CompilationUnit`/`partition_fold`/R4 interface/`Provenance`) landing in-tree (sibling lane, `adhoc-85078326-fdb`),
   - main-green (per the Rust design doc's same gating note — do not scope-creep into unrelated red).
3. **Land order:** C R3 (linkage spelling) can be realized as soon as the home lands, independent of whether the Rust `PartitionPolicy` (crates≈cores derivation) has landed — the C realization only needs `CompilationUnit` values and a `Map`, the same contract the Rust realization consumes (§13 of the Rust design doc: "the `Map` is the only thing that crosses into the shared fold"). C R4 lands alongside R3 (the validity predicate is what makes R3's output provably safe to emit).

## 8. § alignment

- **§2 (Realization):** the whole point of this doc — the same `partition_fold` kernel, two realizations (Rust R3/R4, C R3/R4), no fork of the shared type.
- **§3 (single authority):** C does not redefine `CompilationUnit`/`partition_fold`/`Provenance`; it supplies R3/R4 only, exactly the layer split the Rust design doc locked.
- **§5 (fail-closed):** the C R4 refusals (§4) are typed and located, never a widened "merge the units" or "recompile everything" fallback; the "permissive on cyclicity" behavior is a *correct* R4 answer, not a missing check — cyclicity is not a C error, so refusing it would itself be the false-positive twin of an absorbing fallback.
- **§7 (self-host):** proves the shared shape by running it through a second, structurally different backend before either realization is cemented into the compiler's own emitted output.

## 9. Interface summary (the handoff)

- **Code against (shared home, sibling lane authors):** `CompilationUnit` + `partition_fold(module_dag, Map) -> List<CompilationUnit>` + `Provenance` + the R4 validity-predicate interface.
- **Own (this doc / C realization):** C R3 (`.h`/`.c` split render, `#include`/forward-decl cross-reference render, include-closure identity, `.o`+link artifact render) + C R4 (permissive-cyclic predicate; refuse only on cyclic by-value type containment or split-definition).
- **Do not:** bake acyclicity into the shared shape or into `partition_fold`; hand-set `interface`/`deps`; put a `.o`/header filename in the shared `artifact` field; treat C's "permissive" R4 as "no R4" (the by-value-cycle and split-definition refusals are real and must fire).
- **Prove by execution (§5/§7):** the four discriminating witnesses in §5 — 1-unit, N-unit-acyclic, N-unit-**cyclic** (the load-bearing one), and a genuinely-invalid partition's typed refusal — each run through the real `cc` transport, not asserted structurally.
