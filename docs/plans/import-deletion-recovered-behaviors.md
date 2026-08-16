# Behaviors recovered from the deleted import tests

Status: **design note, nothing implemented.** This exists so that deleting the
tests that asserted import *semantics* does not also delete the *properties*
some of them were the only proof of.

The import statement is gone from the corpus (`^import` count in `.dag` is
zero) and `import` is a parse error. The tests below asserted the mechanism
that is gone, so they were deleted with it. Each entry records the property
worth re-establishing, in namespace terms, once closure assembly is rebuilt.

Nothing here is a plan to restore the deleted mechanism. A property is only
worth re-expressing if it is still true under namespace-only resolution.

---

## 1. Pool membership must not be sufficient for binding

From `class_b_trim_specimen_test` (deleted; it was already a declared scaffold
whose stated dissolution was "delete this Rust module").

**The property:** a bare name must not resolve merely because some module that
happens to define it is present in the compilation pool. Under the import
regime this was proven by the pair "explicit import binds / bare use refuses
even when `std.algebra` is already in the pool".

**Why it survives the deletion of imports.** The refusal arm is the half that
matters and it is not about imports at all: it says binding must follow from a
*declared, derivable* relationship, not from incidental co-presence. That is
the Class-B pool-membership-coincidence class
(`import-strip-witness-discovery-cascade-diagnosis.md`), and this lane makes it
sharper rather than moot — with imports deleted, *every* module's dependencies
are derived rather than declared, so "did this bind for a real reason or a
coincidental one" becomes the central question rather than an edge case.

**Re-expression, once closure assembly is reference-derived:** a name whose
definer is in the pool but is NOT reachable through the reference closure from
the entry must refuse. The positive control is the same name reachable through
a real reference. Note this is only meaningful if the pool can ever be *wider*
than the closure; if assembly makes them identical by construction, the
property becomes structurally impossible to violate (§4b) and needs no test —
which is the better outcome and should be checked for first.

## 2. A unique variant projection resolves without qualification

From `decl_facts_dimensionless_projection_test`'s two `explicit_import`
tests (deleted). Sibling tests in that file survive.

**The property:** where exactly one module in scope exports a given variant
arm, a use of that arm resolves to it unambiguously, and the decl-facts
projection agrees with the resolver about which declaration it found.

**Re-expression:** this is precisely unique-on-chain with a candidate set of
one, so it should hold *more* readily without imports than with them. Worth an
explicit witness because the interesting case is the boundary: exactly one
candidate resolves, two candidates must refuse as ambiguous rather than
first-hit. The surviving `namespace_unique_on_chain_policy_test` cases already
cover the general form; what was import-specific here was only how the single
candidate got into scope.

## 3. The historical ImportScoped policy

From `import_scoped_default_resolves_homonym_fixture_clean` (deleted).

**Not a property to preserve.** This asserted that with the policy bracket set
to `false`, resolution kept the pre-namespace behavior verbatim: nearest-wins
for types, first-hit for functions. First-hit is the silent-pick class that
`NamespaceOnlyY` exists to refuse, and the policy's other arm is already the
default in the executing seed.

Recorded here only so that its deletion is not later mistaken for an accidental
loss of coverage. The behavior it pinned is the one being replaced. The
sibling `namespace_only_refuses_*` tests, which assert the *refusal*, are
retained.

## 4. Ephemeral generated source roots participate in resolution

From `helpers::tests::resolver_imports_ephemeral_generated_source_root`
(deleted — it tested the import-driven closure helper itself).

**The property:** a module written into a temporary generated source root is
found and loaded by closure assembly, not only modules that exist in the
checked-in tree.

**Re-expression:** this is a real requirement of whatever replaces
`extract_imports`, and it is about the *index*, not about imports — the
module-source index must span every configured root including ephemeral ones.
It should be re-asserted directly against the new assembly path.

---

## What is deliberately NOT in this note

The tests that fail today with `unresolved type 'Nat'`, `'FilePath'`,
`'FieldOfFractions'` are **not** listed here and were **not** deleted:
`field_of_fractions_construction_test`, the four
`materialization_provider_resolved_graph_consumer_test` cases, and
`namespace_only_refuses_fn_parent_homonym_at_call_site`.

None of them tests import semantics. They are the evidence that the closure
rebuild worked, and deleting them would remove the only executing check on the
lane's keystone.

**Why they failed, corrected 2026-08-16.** The sentence above previously read
"they fail because closure assembly is currently import-driven, so their
dependencies never load." That diagnosis was superseded by its own lane:
`resolve_imports_transitively_with_source_roots` now delegates to
`closure_for_entry`, which is reference-driven. The real cause was found by
diffing the failing files against main rather than by reasoning further about
the resolver, and it was two things:

1. This cut's corpus pass had stripped `import` lines out of `.dag` fixtures
   that live inside **Rust string literals** and under `dag/test/fixture/`,
   without qualifying what those imports had bound. The string-literal oracle
   did not catch it because the oracle's subject is `.dag` FILES and those
   fixtures are `.dag` CONTENT in a `.rs` file — outside its denominator by
   construction.

2. `closure_inner` looked every referenced name up in an index keyed on
   **simple** declaration names, so a qualified reference
   (`std.algebra.FieldOfFractions`) never matched and the miss arm was a bare
   `continue` — the dependency was silently dropped and the closure came back
   short. Since qualification is what this cut substitutes for `import`, the
   closure builder had no way to follow the edge the cut creates.

The failure shape is worth keeping: bare cross-file names match EVERY declaring
module (the index deliberately refuses to pick a winner), which in a densely
interconnected corpus multiplies closure width — the runtime cost and the crash
are that width, not stack tuning.

**Next-rung trigger for the `continue`.** It is repaired for qualified names and
still silent for genuinely unknown ones, so the class sits at *mitigatable*. It
should become a refusal — but not before the tree's remaining unresolved
population reaches zero, because converting it earlier would require a
suppression list, and a suppression list at a refusal arm is the escape hatch
§5 forbids.
