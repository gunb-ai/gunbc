# The 164 runtime-errored floor identities, partitioned by causal family

**Subject:** required-floor run `32633501354`, main push, head `907f19c2cc`
(`Runner slot membership becomes fleet-converge's third member family`), 2026-08-23T10:20Z.
Its ledger line reports `known_red_runtime_errored=164`; `identities.tsv` is those 164 rows,
one per line, as the run itself printed them.

**Producer:** `gh run view 32633501354 --log`, the `KNOWN-RED-RUNTIME-ERRORED` lines, split on the
run's own `is enrolled as expected-red but RUNTIME-ERRORED, not failed:` separator. Column 4 (the
missing name) is joined against a declaration scan of `dag/` and `src/` at the same head.

Columns: `identity`, `family`, `subclass`, `missing name` (empty where the family has none),
`the run's own message`.

## What every one of the 164 has in common

They are programs the compiler ACCEPTED and the interpreter could not evaluate. None is a
compile refusal, and none is a failed assertion — a claim that threw produced no verdict at all,
which is why the floor refuses to let enrollment hold them.

## The families

| family | identities | what the compiler let through |
|---|---|---|
| `REFERENCE-UNAVAILABLE` | 149 | a name resolved at typecheck that the evaluator cannot bind |
| `HOST-PRIMITIVE-CONTRACT` | 11 | a host primitive called with an argument list its host arm rejects |
| `FIELD-ON-WRONG-TYPE` | 2 | `.raw` on an `Int` |
| `CALL-CONTRACT-MISMATCH` | 1 | a call missing a required argument |
| `UNBOUNDED-RECURSION` | 1 | a divergent chain, refused by the interpreter's depth wall |

The last is not a wall gap: the interpreter refuses it with a typed, located, bounded diagnostic.
It is listed because it is one of the 164, not because it is owed a fixture.

## The reference-unavailable subset, by intended compile state

Every one of the 149 names IS declared somewhere in the corpus. **`R4-undeclared-anywhere` is
empty** — not one of them is a typo. So "intended compile state" never separates *should have
refused* from *should have resolved* on the ground of the name not existing; it separates on
WHERE the declaration lives and WHAT KIND it is.

| subclass | identities | distinct names | the declaration the reference names |
|---|---|---|---|
| `R1-data-in-unimported-module` | 126 | 60 | a module-scope `data` in a module the referring file does not import |
| `R6-variant-or-type-name` | 9 | 3 | a coproduct variant / type name used as a bare value |
| `R2-fn-in-unimported-module` | 8 | 3 | a `fn` in a module the referring file does not import |
| `R3-test-decl-in-test-module` | 5 | 5 | a `test data` declared in another `*_test.dag` |
| `R5-type-only` | 1 | 1 | a type name (`Dag`) in call position |

R1 is the class, and it is 85% of the reference-unavailable subset.

## R1 measured end to end, with its discriminating control

Executed at head `907f19c2cc` with a release `gunbc` built from that tree (BuildBuddy, one
dispatch; the session's preinstalled `/usr/local/bin/gunbc` is a stale vintage and refuses the
current `dag/std/algebra.dag` at parse, so it was not used):

| probe source | compile | evaluation |
|---|---|---|
| `fn go() { argument_form_is_valid(design_argument) }`, no imports at all | accepted, silent | `NoSuchFunction { name: "design_argument" }` |
| the same call with `import gunbc.design_argument { design_argument }` | accepted | evaluates, `true` |

The import is the whole discriminator, and the compiler says nothing about its absence. The
second row is what makes the first a finding rather than a broken fixture: the declaration is
fine, the reference is fine, and only the loader's reach differs.

## Why no R1 control fixture is in this PR, stated as a measurement rather than as a plan

The two `.dag`-callable compile-observation surfaces both REFUSE the R1 shape, so a fixture built
on either would be permanently green and would be cited as coverage of a class it never touches
(DESIGN §4b: a check whose RED is unauthorable is worse than absent).

| surface | R1 shape (`bare data reference`) | bare name declared nowhere | clean control | broken control |
|---|---|---|---|---|
| `compile_dag_diagnostic_census` | `InternalError variable:design_argument`, blocking | `InternalError`, blocking | 0 rows | — |
| `compile_dag_rust_emit_check` | `false` (refuses) | `false` (refuses) | `true` | `false` |
| the floor's own witness loader (production) | **accepted** | — | — | — |

Both surfaces compile a virtual source through import-closure discovery. The floor's witness
loader additionally runs `cli_run` `extend_with_bare_reference_closure`, which resolves a bare
name through the tree census and pulls the module it names. That closure is why the reference
typechecks in production, and the interpreter then binds functions and not module-scope `data`.

**Next-rung trigger for the R1 control:** a compile-observation surface that resolves names by the
witness loader's rule — the bare-reference closure — rather than by import-closure discovery
alone. Until one exists, the class is observable only as the floor's own
`known_red_runtime_errored` counter, and that counter goes to zero as the 149 are repaired.
`gunbc#9006` is repairing 21 of them (the `*_published_mock_corpus` names) as this is written.

## What IS in this PR

`HOST-PRIMITIVE-CONTRACT` reproduces on `compile_dag_rust_emit_check` — the surface accepts
`atom_identity_hash("a", "b")` and the host arm refuses it at evaluation. That is an authorable
RED, and it is the fixture this PR lands:
`dag/test/claim/compile_accepted_unevaluable_program_control_test.dag`.
