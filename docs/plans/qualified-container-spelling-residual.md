# Declared residual: qualified container spellings answer rather than refuse

**Status:** rung stated with its trigger (DESIGN §4b(3)). Not a fix in progress — a known
ceiling, declared so that it is not rediscovered by breakage a third time.

## What the cut creates

`origin/main` contains **zero** occurrences of `std.types.List<`, `std.types.Map<` or
`std.types.Set<`. The namespace cut replaces bare cross-module references with qualified ones, so
qualified container spellings are a **branch-introduced input shape**. Several container lookups
in `std.types` are keyed on the authored spelling, and several call sites pass the FULL authored
name rather than its last segment — `v1.compiler.trait_derive_emit v1_type_expr_is_keyed_map`,
`v1.compiler.infer_types is_fully_resolved` — so each qualified spelling needs its own row.

This requirement was previously discovered and satisfied **one member at a time**: branch commit
`c31ff08d` added the `std.types.List` rows when List broke. Set and Map broke on 2026-08-21 and
were added then. Treating it as a class instead surfaced five more that had never been exercised.

## Population closed on 2026-08-21

Derived rows, computed from the bare row each mirrors rather than hand-authored — 9 rows across
3 tables, covering 5 spellings / 345 sites:

| spelling | sites | arity | template algebra | canonical |
|---|---|---|---|---|
| `std.algebra.FreeMonoid` | 189 | — | yes | yes |
| `v2.std.witness.Witness` | 145 | yes | — | yes |
| `std.algebra.BooleanAlgebra` | 9 | — | yes | yes |
| `std.algebra.PointwisePower` | 1 | — | yes | **no** |
| `std.algebra.PartialFunction` | 1 | — | yes | yes |

The two asymmetries are load-bearing evidence that the rows were **derived, not authored**:
`Witness` takes an arity row and no template row, and `PointwisePower` takes a template row and
**no** canonical entry — because its bare name is absent from `canonical_container_names`. Written
by hand, both would have been filled in by symmetry and been silently wrong.

`ordered_element_collections` deliberately received nothing: it carries `List` only, because Set
and Map are unordered. Adding them there would have passed every check and been quietly incorrect.

## The residual

**The class answers; it does not refuse.** Both consuming arms are `Absent => false`:

- `v1_type_expr_is_keyed_map` — a missing row reads as *"this is not a keyed map"*
- `is_fully_resolved` — a missing row reads as *"this is not under-parametrized"*

So a **sixth** qualified container spelling added tomorrow will silently answer `false` and
produce no evidence of itself. That is ⊥-as-ignorance rendered as ⊥-as-answer (DESIGN §5,
empty-observation narrow). Today's population was only loud because the `set_contains` receiver
check happens to refuse; the two arms above would have gone on answering wrongly indefinitely.

**Rung: mitigatable.** The invalid state remains writable and nothing detects it.

## Why the wall is not built here

Converting `Absent` to a refusal **wholesale is wrong**, and this is measured, not assumed:
`container_type_arity` has 7 keys while the corpus contains **36,311** distinct type-position
names, so **36,304 of them reach `Absent` correctly**. `String`, `Bool` and every user record are
not containers. The narrowing is real only for the input class "qualified spelling of a known
container".

The correct trigger is therefore three conjuncts — leaf is a known container name, AND the full
name has no row, AND the name is **not** `<known-service>.<Operation>`. The third is required:
six sites in the tree have a container leaf and are **not** containers, all service operations —
`github.Pulls.List` (3), `os.Hostname.Set` (2), `cron.Tab.List` (1). Without it the wall refuses
those six and the remedy it prints — *add a row* — would declare `github.Pulls.List` a container
of arity 1. Loud, pointing the wrong way.

Implementation is not contained even though the blast radius is: neither arm has a refusal
channel (both are pure `Bool` predicates), `v1.compiler.infer_types` raises no diagnostics at all,
`std` has no refusal primitive, and `is_fully_resolved` has 13 callers. That is a
diagnostic-threading refactor, and absorbing it into an import cut is the scope error DESIGN §6
warns about.

## Restoration trigger

Lane `adhoc-7090cf8f-d92` — convert both arms to a typed, located, counted refusal under the
three-conjunct trigger above. The six service-operation sites are its **discriminating RED**: the
wall must distinguish them from the 345 legitimate spellings rather than merely firing. When that
lands, this residual closes and the refusal stays enrolled as the permanent instrument (§4b(4)) so
a sixth spelling announces itself instead of answering `false`.

## What this declaration does NOT claim

It does not claim the 345 sites were producing wrong output — only wrong *answers to these two
predicates*, whose downstream effect was not traced. It does not claim the five spellings are the
complete set for all time; they are the complete set **measured in this tree on 2026-08-21** by a
string-masked scan. And it does not claim the rows are verified by execution beyond regen reaching
zero hard diagnostics with them in place.
