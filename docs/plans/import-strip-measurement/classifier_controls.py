#!/usr/bin/env python3
"""Discriminating controls for the residual classifier's named dispositions.

The hygiene bucket is asserted at ZERO. A classifier rule that can quietly move
a row OUT of that bucket is therefore aimed straight at the claim under test, so
each named disposition is checked here in BOTH directions: the named subject
gets its disposition, and a planted subject that merely LOOKS like it does not.

Run:  python3 classifier_controls.py     (exit 0 = all controls hold)
"""
import sys
import os

sys.path.insert(0, os.path.dirname(os.path.abspath(__file__)))
from classify_residual import disposition, classify_rows  # noqa: E402

FAILURES = []


def check(label, got, want):
    if got != want:
        FAILURES.append(f"{label}: got {got!r}, want {want!r}")
    print(f"{'ok  ' if got == want else 'FAIL'} {label}: {got}")


# --- control 1: a planted accidental fork with a CONVENTION-sized population ---
# Ten declarations of an unrelated name. The retired rule returned
# per_module_convention_population on count alone, so this row vanished from the
# hygiene count. It must stay hygiene.
check("planted 10-declaration accidental fork stays hygiene",
      disposition("plausible_helper", "callee", 10, False, "",
                  candidates=[f"gunbc.module_{i}" for i in range(10)]),
      "corpus_hygiene")

# ...and a real convention name whose population is real still classifies.
check("named convention subject keeps its disposition",
      disposition("extdeps_model_scope", "callee", 94, False, "",
                  candidates=[f"extdeps.pkg_{i}" for i in range(94)]),
      "per_module_convention_population")

# ...but the SAME name declared twice inside one module is a fork, not a
# convention: the property is one-per-module, and it no longer holds.
check("convention name violating one-per-module is not excused",
      disposition("extdeps_model_scope", "callee", 3, False, "",
                  candidates=["extdeps.a", "extdeps.a", "extdeps.b"]),
      "duplicate_unclassified")

# --- control 2: an unknown duplicate across two fixture modules ---
# The retired rule excused any duplicate whose every candidate path contained
# `.fixture.`. It must stay hygiene.
check("unknown duplicate across two fixture modules stays hygiene",
      disposition("unrelated_dupe", "callee", 2, False, "",
                  candidates=["test.fixture.alpha", "test.fixture.beta"]),
      "corpus_hygiene")

# --- control 3: the named intentional-ambiguity subjects ---
check("named intentional-ambiguity subject keeps its disposition",
      disposition("SharedBareArm", "type", 2, False, "",
                  candidates=["test.fixture.decl_facts_reflection.ambiguous_shared_a",
                              "test.fixture.decl_facts_reflection.ambiguous_shared_b"]),
      "intentional_ambiguity_fixture")

# ...pinned to the EXACT pair: a third declaration elsewhere is not excused.
check("intentional-ambiguity name with a third declaration is not excused",
      disposition("SharedBareArm", "type", 3, False, "",
                  candidates=["test.fixture.decl_facts_reflection.ambiguous_shared_a",
                              "test.fixture.decl_facts_reflection.ambiguous_shared_b",
                              "gunbc.somewhere_real"]),
      "duplicate_unclassified")

# --- control 4: the SAME property, exercised through the PRODUCTION path ---
# Controls 1-3 call `disposition()` directly. That is how the raw-population
# defect survived them: the production builder deduplicates candidate modules
# into a set before `disposition()` ever sees them, so a convention name
# declared TWICE IN ONE MODULE arrived as a single-module list and satisfied
# one-per-module by construction. A control that reaches only the predicate
# proves the predicate. This one goes through `classify_rows`, which is the
# seam the ledger is actually built from.

def disposition_via_production_path(name, decl_sites):
    line = f"function '{name}' not found in scope (dag/x.dag:1-2)"
    rows = classify_rows([line], {name: decl_sites}, {})
    return rows[0]["disposition"]


check("PRODUCTION PATH: convention declared twice in ONE module is not excused",
      disposition_via_production_path(
          "extdeps_model_scope",
          [("extdeps.a", "data"), ("extdeps.a", "data"), ("extdeps.b", "data")]),
      "duplicate_unclassified")

check("PRODUCTION PATH: genuine one-per-module convention still classifies",
      disposition_via_production_path(
          "extdeps_model_scope",
          [(f"extdeps.pkg_{i}", "data") for i in range(94)]),
      "per_module_convention_population")

check("PRODUCTION PATH: planted accidental fork stays hygiene",
      disposition_via_production_path(
          "plausible_helper", [(f"gunbc.module_{i}", "fn") for i in range(10)]),
      "corpus_hygiene")

# The ledger's DISPLAY columns must stay distinct-module even though the check
# now receives the raw population - the repair separates two facts, it does not
# swap one for the other.
_rows = classify_rows(
    ["function 'extdeps_model_scope' not found in scope (dag/x.dag:1-2)"],
    {"extdeps_model_scope": [("extdeps.a", "data"), ("extdeps.a", "data"),
                             ("extdeps.b", "data")]}, {})
check("ledger candidate columns remain DISTINCT modules",
      (_rows[0]["candidate_provider_modules"], _rows[0]["candidate_count"]),
      ("extdeps.a|extdeps.b", 2))

if FAILURES:
    print(f"\n{len(FAILURES)} control(s) FAILED")
    sys.exit(1)
print("\nall controls hold")
