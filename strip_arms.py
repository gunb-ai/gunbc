#!/usr/bin/env python3
"""Derive an ARM-FREE 04_infer.dag from the head being measured, instead of from a committed snapshot.

Why: probe v8 installed a COMMITTED carrier (carriers/A_infer_merged.dag) as the arm-free tree. That snapshot
predated the variant revert -- it still used TextNotAskedForCallableComponentResidue, which #14287 deleted on
main and stern-bat-401 ordered removed. The regen therefore refused the whole tree at round 1 with 'not found
in scope' and the probe died having measured nothing. A committed snapshot of a moving source is a stale
carrier; deriving from the head under test means the two cannot drift apart.

Removal is CHARACTER-based, not line-based, because in this chain an arm's closing brace shares its line with
the next `else`: `} else {`. Deleting whole lines would take the `else` with them and leave a file that
cannot parse. So: find the arm's opening token, keep its leading `}`, and delete up to and including the
brace that closes the arm body.

Every guard below is an assertion and nothing is written unless all pass. A silently partial strip would
compile and measure the wrong thing, which is worse than a refusal.
"""
import sys, re

def die(m):
    print("STRIP REFUSED: " + m); sys.exit(44)

src, dst = sys.argv[1], sys.argv[2]
text = open(src, encoding="utf-8").read()
if "UnifyGenericsAcc" not in text:
    die("no UnifyGenericsAcc -- this is not the fold's file")
bad = [i + 1 for i, l in enumerate(text.split("\n"))
       if "TextNotAskedForCallableComponentResidue" in l and not l.lstrip().startswith("//")]
if bad:
    die("source USES the deleted variant at lines %s; measure a head that no longer does" % bad[:5])

before = text.count("UnifyGenericsAcc")
# The source file's own brace balance is NOT necessarily zero (braces appear inside string literals and
# comment prose), so the guard below is a DELTA on the balance, never an absolute zero test.
balance_before = text.count("{") - text.count("}")

# --- (a) the two arm call sites, character-based ---
removed_arms = 0
while True:
    tok = re.search(r"\} else if binding_is_generic_declaration_own_form\(", text)
    if not tok:
        break
    i = tok.end()                      # just after the opening '('
    # skip to the '{' that opens the arm body
    while i < len(text) and text[i] != "{":
        i += 1
    if i >= len(text): die("arm body never opens")
    depth = 0; j = i
    while j < len(text):
        if text[j] == "{": depth += 1
        elif text[j] == "}":
            depth -= 1
            if depth == 0: break
        j += 1
    if depth != 0: die("arm body never closes")
    text = text[:tok.start() + 1] + text[j + 1:]   # keep the leading '}', drop `else if ... { body }`
    removed_arms += 1
    if removed_arms > 4: die("more than 4 arm sites -- the pattern is matching something unintended")
if removed_arms != 2:
    die("expected exactly 2 arm call sites, removed %d" % removed_arms)

# --- (b) the two predicate definitions, with the comment block above each ---
for name in ("node_mentions_own_parameter", "binding_is_generic_declaration_own_form"):
    lines = text.split("\n")
    heads = [i for i, l in enumerate(lines) if re.match(r"^fn %s\(" % name, l)]
    if len(heads) != 1:
        die("expected exactly 1 definition of %s, found %d" % (name, len(heads)))
    h = heads[0]
    k = h - 1
    while k >= 0 and (lines[k].lstrip().startswith("//") or lines[k].strip() == ""):
        if lines[k].strip() == "" and (k - 1 < 0 or not lines[k - 1].lstrip().startswith("//")):
            break
        k -= 1
    depth = 0; started = False; end = None
    for j in range(h, len(lines)):
        depth += lines[j].count("{") - lines[j].count("}")
        if "{" in lines[j]: started = True
        if started and depth == 0:
            end = j; break
    if end is None: die("definition %s never closes" % name)
    del lines[k + 1:end + 1]
    text = "\n".join(lines)

if "binding_is_generic_declaration_own_form" in text or "node_mentions_own_parameter" in text:
    die("strip left residue")
after = text.count("UnifyGenericsAcc")
if after != before - 2:
    die("expected to remove exactly 2 fold occurrences (the two arms), removed %d" % (before - after))
balance_after = text.count("{") - text.count("}")
# Each removal takes an arm's own '{' and its matching '}' together, so the DELTA must be 0: a non-zero
# delta means a half-arm went (a brace without its partner), i.e. the chain was cut in the wrong place.
# The absolute balance is not zero in this file (a brace sits inside string/comment text), which is why
# this is a delta test -- an absolute test refused a correct strip.
if balance_after != balance_before:
    die("strip changed brace balance from %d to %d -- a brace lost its partner, so an arm was cut in the "
        "wrong place" % (balance_before, balance_after))
for must in ("} else {", "type mismatch", "later argument", "fn unify_generics("):
    if must not in text:
        die("strip removed fold machinery: %r gone" % must)
open(dst, "w", encoding="utf-8").write(text)
print("STRIP OK: fold occurrences %d -> %d (2 arms removed); %d lines remain; brace-balanced"
      % (before, after, text.count("\n") + 1))
