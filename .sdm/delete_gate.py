#!/usr/bin/env python3
"""Treatment B: delete the coarse outer gate, per parent's item 4.

`call_needs_generic_rebinding_pass` and its helper `expr_contains_lambda` are removed
from src/v1/04_infer.dag; the sparse fold's `still_missing_after` decision
(`argument_needs_generic_recheck`) becomes the sole authority for whether an argument
is re-derived.

Fail-closed (DESIGN.md sec.5): every anchor must match exactly once or this aborts
without writing. Deleting the wrong text would silently change what is measured.
"""
import sys, re

P = "src/v1/04_infer.dag"

GATE_IF = """  if !call_needs_generic_rebinding_pass(call_args: call_args, generic_names: generic_names, subst: discovered.subst) {
    first_state
  } else {
"""
CLOSE = """      }
    )
  }
}
"""
DELETED_FNS = """fn expr_contains_lambda(n: Node) -> Bool {
  is_lambda_expr(e: n) || (n.children |> any(c => expr_contains_lambda(n: c)))
}
"""
GATE_FN_START = "// Second pass of a call is admitted only when the old trigger would have run one:"
GATE_FN_END = """fn call_needs_generic_rebinding_pass(call_args: List<Node>, generic_names: List<String>, subst: Map<String, Node>) -> Bool {
  if generic_names |> count == 0 {
    false
  } else if map_is_empty(subst) {
    false
  } else {
    call_args |> any(a => expr_contains_lambda(n: arg_value(n: a)))
  }
}
"""


def die(m):
    sys.stderr.write("DELETE-GATE FAIL: %s\n" % m)
    sys.exit(7)


def main():
    s = open(P).read()

    if s.count(GATE_IF) != 1:
        die("gate-if anchor matched %d" % s.count(GATE_IF))
    if s.count(CLOSE) != 1:
        die("fold-close anchor matched %d" % s.count(CLOSE))

    # 1) make the fold unconditional: drop `if !gate { first_state } else {`
    s = s.replace(GATE_IF, "  ", 1)
    #    and drop the one extra closing brace the else-block needed
    s = s.replace(CLOSE, "      }\n    )\n}\n", 1)

    # 2) delete the two definitions
    i = s.find(GATE_FN_START)
    j = s.find(GATE_FN_END)
    if i < 0 or j < 0:
        die("gate definition block not found")
    s = s[:i] + s[j + len(GATE_FN_END):]
    if DELETED_FNS in s:
        die("expr_contains_lambda definition still present")

    # 3) no dangling references
    for name in ("call_needs_generic_rebinding_pass", "expr_contains_lambda"):
        n = s.count(name)
        if n:
            die("dangling reference to %s (%d)" % (name, n))

    # 4) the authority we are betting on must still be there
    for name in ("argument_needs_generic_recheck", "still_missing_after"):
        if name not in s:
            die("authority %s disappeared" % name)

    open(P, "w").write(s)
    print("DELETE-GATE OK: gate + helper removed, sparse fold is the sole authority")


main()
