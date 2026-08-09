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
from classify_residual import disposition  # noqa: E402

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

if FAILURES:
    print(f"\n{len(FAILURES)} control(s) FAILED")
    sys.exit(1)
print("\nall controls hold")
