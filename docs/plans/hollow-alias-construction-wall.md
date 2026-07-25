# Design — the hollow-alias construction wall

**Status:** DRAFT, design-note-first per sharp-bee-290 mandate `msg_7a83651e-f370-4b2a-8786-e5c0d81aeaa5` (nod 2). Capstone of the completion-pattern lane — the operator sequenced the wall behind the GroupCompletion/FieldOfFractions specimens (PR #7197, infra-blocked with the operator; PR #7210, landed §7.2 model grounding — that PR's own design note was authored on the #7197 branch and has not merged to main, so it is not linked here to avoid a dangling reference), not behind their full implementation, so this note can proceed while `FieldOfFractions` realization (§7.3 there) is still held for sign. Not implemented; no code lands from this note yet.

Owner: eager-crane-304. House method: press-fix the specimen, then write the law that makes the whole class unwritable (the same shape as the frontend lane's press-fix → interaction-totality move).

## 1. The class, generalized from two specimens

`dag/std/algebra.dag:38`'s `GroupCompletion<M>` (fixed, PR #7197) and `dag/std/algebra.dag:40`'s `FieldOfFractions<R>` (model fix landed, PR #7210, §7.2 of that unmerged design note) were the same defect shape: a `type Name<Params>` declaration with **no RHS at all** — no `{ field: T, ... }` record body, no `= Alias` alias RHS, no `= Variant | Variant` coproduct RHS — reached at a **construction or emit position** (a record literal built against it, or a value of that type rendered to target code). Confirmed root cause (traced in the GroupCompletion note, §7.1.1): a bodyless generic type renders in `src/v1/05_emit_rust.dag` as `std::marker::PhantomData<...>` (`rust_phantom_marker_inner`/`rust_phantom_field_name`, lines 4378/4594/4603/4634, and the zero-field unit-struct branch at `emit_struct_from_children` line 4627-4628) — a marker carrying **no data**, so any value of that type silently has nothing behind it. This is a §5 fail-open with a compile-time-decidable trigger: the emitter did not refuse, it fabricated a plausible-looking empty struct.

This is exactly the pattern DESIGN §5 names: "the deepest trap is specification-without-execution... treat your own output as unverified until a consumer runs it green" — a `type GroupCompletion<M>` declaration typechecks fine (there is no rule against a bodyless declaration existing), so the anemia was invisible to every consumer except the one that actually ran the emitted code (`cargo build`, the deep-seven probe). The operator's sharpening: this class is **construction-decidable**, not merely detectable-by-execution — a lens (or, better, a typechecker refusal) can catch it *before* emission, the same way §5's "correctness by construction, not validation" principle asks for.

## 2. Census — and why grep is the wrong tool for it (a self-demonstrating finding)

A naive text census (`grep -rn "^type [A-Z][A-Za-z0-9_]*<[^>]*>$\|^type [A-Z][A-Za-z0-9_]*$"` filtered for lines with no trailing `= `) returns **1,739 hits** across `dag/` and `src/v2/`. This number is **wrong and must not be cited** — inspection of the first handful shows the overwhelming majority are multi-line coproduct declarations where the `=` sits on the *next* line:

```
type JobserverCitation
  = JobserverCited { config: JobserverConfig, provenance: String }
  | JobserverAbsent
```

(`dag/extdeps/jobserver.dag:35-37` — genuinely bodied, not hollow; the grep only inspected the `type` line in isolation.) This is itself a live instance of the recurring failure mode this repo's memory already names — **"grep oracle misses termination"** / **"a lens reads shape, not contents"** — applied one level up: a *census* claiming to measure a construction-decidable class needs the same structural read the wall's own predicate needs (parse the declaration's RHS shape, not grep the line it starts on). No corpus-wide hollow-declaration count is asserted in this note; the wall's own construction (§4) is what makes the count trustworthy, and only that count should ever be cited.

Two real (structurally-verified, not grepped) examples found by hand:

- **Genuine hollow, currently harmless:** `dag/extdeps/hardware.dag:13` — `type Hardware` has no RHS at all, but is used exactly once in the whole corpus, `cpu_vendor(cpu: CpuFacts) -> Vendor<Hardware>` (`dag/extdeps/cpu.dag:11`) — purely as a **generic type-tag parameter**, never as the target of a record literal, never rendered as its own emitted value. This is the phantom-type-as-domain-discriminator pattern (cf. Rust's own idiomatic `PhantomData<Marker>` tagging) and is legitimately fine — the wall must not refuse it, because it is never *construction-reached* (§3).
- **Genuine hollow, was harmful:** `GroupCompletion<M>`/`FieldOfFractions<R>` — reached at construction (`GroupCompletion{pos, neg}` record literals exist in the corpus once `Int`/`Rational` values are built) and at emit (both are the generic parameter of `Int`/`Rational`, which is emitted pervasively) — this is the class that actually broke.

The distinguishing fact between these two is not "is the declaration hollow" (both are) — it is **whether the hollow type is ever construction-reached**. That is the wall's actual predicate.

## 3. The frontier — reusing an existing idiom, not inventing one

Sharp-bee-290's sharpening asks for one honest frontier: "genuinely-abstract opacity is legitimate... the wall needs a DECLARED-ABSTRACT annotation." Searching the corpus for an existing idiom before inventing new syntax (DESIGN §2 decomposition discipline — DFS the concept DAG first) finds one already in live use, 11 occurrences corpus-wide:

```
type VerifyCheck = Node
type ResolvedTree = Node
type CoreNode = Node
type ParseTree = Node
...
```

(`dag/std/patterns.dag:4-6`, `src/v2/compiler/03_resolve.dag:84`, `src/v2/compiler/00_compile.dag:59,61`, `src/v2/compiler/normalized_tree.dag:5`, `src/v2/compiler/06_translate.dag:195`, `src/v2/compiler/self_host.dag:35`, `src/v2/std/grammar.dag:325`, `src/v2/lens/affected_set.dag:65`.) An existing design doc (`docs/plans/realization-measurement-loop.md`) already names this pattern explicitly: *"map→reduce execution decomposition is forward-stubbed (`= Node`) — anticipates sharding."* `= Node` is a **real alias**, not a hollow declaration — it has an RHS, and that RHS is the substrate's own untyped primitive (§4 of DESIGN.md: "a program is a dependency graph over two primitives"). Constructing a value of a `= Node`-aliased type is legitimate (it constructs a `Node`); the type is deliberately left unrefined, not accidentally left empty.

**This is the frontier, reused rather than minted:** a "declared-abstract" type is one whose RHS is (transitively) `Node` — the corpus's own idiom for "deliberately unrefined, not yet modeled." A **genuinely hollow** type is one with *no RHS at all*. The wall's rule:

> **Bodyless (no RHS at all) + construction-reached ⇒ refuse. Bodyless + `= Node`-aliased (declared-abstract) ⇒ legitimate, not walled. Bodyless + never construction-reached (pure type-tag, like `Hardware`) ⇒ legitimate, not walled.**

No new annotation syntax is proposed. The existing `= Node` idiom already carries the honest-opacity marker; inventing a second one (a doc-comment tag, a new keyword) would itself be a §3 fork of a concept the corpus already names.

## 4. Where the refusal lives — lens first, promotion path named up front

Per DESIGN §6 ("construction first... reserve the lens for the unstructurable residue" — but also "a lens is validation... it concedes the bad state is writable"), the honest sequencing is:

1. **Lens now:** a pure reader over the `Node` tree — for every `type Name<Params>` declaration with no RHS and not `= Node`-aliased (transitively), check whether `Name` is ever the type of a record-literal construction (`Name{...}` node) or ever reaches the emitter's type-rendering path for a *value position* (not merely as a phantom generic argument to another type). Both checks are structural (walk the tree for `RecordLit` nodes whose declared type resolves to `Name`; walk emitted-type call sites), matching "a lens reads shape, not contents." This is the residue mechanism, proven against the corpus (the census in §2, done properly this time — a real parse, not grep) before anything promotes.
2. **Promotion to typechecker refusal, once proven:** once the lens has run clean against the whole corpus (zero false-refusals on legitimate `= Node`/tag-only types, and it would have caught both `GroupCompletion` and `FieldOfFractions` pre-fix as discriminating RED), the same predicate becomes a typed compile refusal — the class becomes genuinely *unwritable*, not merely *lint-flagged*. This mirrors the operator's own framing: "whether the refusal lives in the typechecker or STARTS as a lens that promotes to the typechecker is exactly what your design note decides" — decision: **lens first**, named promotion trigger = zero-false-positive clean run across the full corpus census.

The refusal, once it fires (lens or typechecker), is typed and located per DESIGN §5 — `HollowConstructionRefused { type_name, declared_at, construction_site }` — never a warning, never an absorbed default.

## 5. A third specimen — the checkpoint_scalar_phantom class (bodyless-alias-in-construction, converging findings)

A related but distinct failure surfaced in the same emitter neighborhood, worth citing here as a **prime bodyless-alias-in-construction case** even though it is not itself the two specimens' exact shape (a hollow `type` declaration) — it is the *emit-side dual*: a **checkpoint scalar's phantom type-parameter widen**. Three independent findings converge on the same root:

1. **The known widen, already named in-code** — `rust_checkpoint_scalar_phantom_params_note` (`src/v1/stage0/src/v1_compiler_emit_rust.rs:746`) documents a live construction wall for E0109 ("type arguments are not allowed on builtin type `i64`"): a checkpoint scalar (`lookup_checkpoint` / `rust_seed_host_numeric_alias` — `Int`, `Nat`, `i64`, etc.) has Rust arity 0, but DAG phantom params from `GroupCompletion<Nat>` or `Compose<Int, MachineWidth<>>` can surface as `i64<T>` if not refused at the type-node renderer. The comment states the root fix explicitly ("refuse phantom arg emission at the type-node renderer when the leaf is a checkpoint scalar and children > 0 — not a post-hoc string strip") but the corpus comment is itself the marker that this refusal is *declared*, not yet *proven wired* everywhere a checkpoint scalar can appear generically-applied.
2. **vivid's `checkpoint_scalar_phantom` bucket** — an independent audit pass bucketed ~50 distinct `E0107` ("wrong number of generic arguments") emitter failures under this same root: a checkpoint-scalar leaf receiving phantom type arguments it cannot syntactically carry in Rust.
3. **loyal-raven's dotted-path finding** — a probe-generated emission surfaced `GroupCompletion<v2.std.nat.Nat>` as a literal dotted qualified-path in Rust *generic position*, producing `E0308` (mismatched types) rather than E0109/E0107 — the same underlying phantom-arg-leak class, but manifesting as an unqualified-vs-qualified-path mismatch instead of an arity mismatch, because the leaf resolution disagreed on whether `Nat` was the bare checkpoint or its fully-qualified DAG name.

These three are the **same defect class as §1–§4**, just observed at the opposite end of the same pipeline: §1–§4 is *construction* (a record literal built against a hollow type) and this is *emission* (a checkpoint-scalar leaf's phantom generic argument, carried through from exactly the same `GroupCompletion<M>`/`Compose<Int, MachineWidth<N>>` construction-pattern shapes named in DESIGN §2's Realization example). Both are instances of a bodyless-or-phantom carrier reaching a position where the target language demands real structure and gets none — the emitter fabricates a plausible-but-wrong rendering (a phantom `<T>` on a zero-arity scalar, or a dotted path where a bare identifier is expected) instead of refusing. This is named here as a **convergence receipt**, not a scope expansion: the wall this note designs (§4) is scoped to hollow *declarations* reached at construction; the checkpoint-scalar-phantom class is the **companion emit-side wall**, tracked as its own root (Root-4, sharp-bee-290-owned, separately gated) rather than folded into this note's lens. Citing it here satisfies the operator's ask to name the convergence and keep the two walls' relationship visible without merging their scopes.

## 6. Discriminating RED (to be written when the lens lands — none exists yet, this is a note)

- **Positive control (should refuse):** a synthetic fixture `type Foo<T>` with no RHS, constructed as `Foo{x: 1}` — lens must flag it. The pre-fix `GroupCompletion{pos, neg}` shape (recoverable from PR #7197's pre-fix diff) is the real-world instance of this control.
- **Negative control 1 (should NOT refuse):** `type Hardware` used only as `Vendor<Hardware>` — never construction-reached — must stay clean.
- **Negative control 2 (should NOT refuse):** `type CoreNode = Node` — declared-abstract via the existing idiom — must stay clean even though `CoreNode{...}`-shaped literals may exist (they construct real `Node` values).
- **Regression control:** rerun against `GroupCompletion`/`FieldOfFractions` post-fix (real bodies) — lens must report clean.
- **Census-honesty control:** the lens's own reported count, cross-checked by hand against a small sample, must not reproduce the §2 grep false-positive rate (a lens that also grep-matches the `type` line in isolation would just relocate the same bug into "lens" clothing).

## 7. Scope boundary

This note does not implement the lens, does not touch the emitter's `PhantomData` rendering path or the checkpoint-scalar-phantom emit wall (§5, Root-4), and does not run the corpus census. It is the design-note-first deliverable the operator asked for as the sequencing gate before implementation starts. Next step on operator go-ahead: implement the lens (§4.1), run it against the full corpus, report the *real* count (replacing the disclaimed §2 number), and name the promotion trigger's actual measured proof.
