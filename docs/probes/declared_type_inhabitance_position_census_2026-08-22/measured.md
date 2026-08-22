# Measured verdicts

Binary: `cargo build --release --bin gunbc` at `abf7194e2b2`, BuildBuddy runner, two independent
dispatches (the second adds the callable-type PARAMETER arm; every other cell is identical across
both, which is the only reason a single-run table is quoted below).

Invocation per arm: `gunbc compile --source-root <arm> --entry <arm>/probe.dag --output-dir /tmp/o --dry-run`.
ACCEPTED means the run printed `compiled: N files emitted, 0 diagnostics`.

```
BEGIN-TABLE
arg_nega                 REFUSED   compile of /root/workspace/repo-root/probe_arms/arg_nega/probe.dag produced 1 hard diagnostic(s):
arg_negb                 ACCEPTED
arg_pos                  ACCEPTED
arg_reach                REFUSED   compile of /root/workspace/repo-root/probe_arms/arg_reach/probe.dag produced 1 hard diagnostic(s):
callableparam_nega       ACCEPTED
callableparam_negb       ACCEPTED
callableparam_pos        ACCEPTED
callableparam_reach      REFUSED   compile of /root/workspace/repo-root/probe_arms/callableparam_reach/probe.dag produced 1 hard diagnostic(
cast_nega                ACCEPTED
cast_negb                ACCEPTED
cast_pos                 ACCEPTED
cast_reach               REFUSED   compile of /root/workspace/repo-root/probe_arms/cast_reach/probe.dag produced 1 hard diagnostic(s):
data_nega                ACCEPTED
data_negb                ACCEPTED
data_pos                 ACCEPTED
data_reach               REFUSED   compile of /root/workspace/repo-root/probe_arms/data_reach/probe.dag produced 1 hard diagnostic(s):
field_nega               REFUSED   compile of /root/workspace/repo-root/probe_arms/field_nega/probe.dag produced 1 hard diagnostic(s):
field_negb               ACCEPTED
field_pos                ACCEPTED
field_reach              REFUSED   compile of /root/workspace/repo-root/probe_arms/field_reach/probe.dag produced 1 hard diagnostic(s):
generic_nega             ACCEPTED
generic_negb             ACCEPTED
generic_pos              ACCEPTED
generic_reach            REFUSED   compile of /root/workspace/repo-root/probe_arms/generic_reach/probe.dag produced 1 hard diagnostic(s):
lambdaret_nega           ACCEPTED
lambdaret_negb           ACCEPTED
lambdaret_pos            ACCEPTED
lambdaret_reach          REFUSED   compile of /root/workspace/repo-root/probe_arms/lambdaret_reach/probe.dag produced 1 hard diagnostic(s):
let_nega                 ACCEPTED
let_negb                 ACCEPTED
let_pos                  ACCEPTED
let_reach                REFUSED   compile of /root/workspace/repo-root/probe_arms/let_reach/probe.dag produced 1 hard diagnostic(s):
listelem_nega            ACCEPTED
listelem_negb            ACCEPTED
listelem_pos             ACCEPTED
listelem_reach           REFUSED   compile of /root/workspace/repo-root/probe_arms/listelem_reach/probe.dag produced 1 hard diagnostic(s):
mapkey_nega              REFUSED   module index refused: 1 unparseable .dag source(s)
mapkey_negb              REFUSED   module index refused: 1 unparseable .dag source(s)
mapkey_pos               ACCEPTED
mapkey_reach             ACCEPTED
mapval_nega              ACCEPTED
mapval_negb              ACCEPTED
mapval_pos               ACCEPTED
mapval_reach             REFUSED   compile of /root/workspace/repo-root/probe_arms/mapval_reach/probe.dag produced 1 hard diagnostic(s):
paramdefault_nega        ACCEPTED
paramdefault_negb        ACCEPTED
paramdefault_pos         ACCEPTED
paramdefault_reach       ACCEPTED
return_nega              ACCEPTED
return_negb              ACCEPTED
return_pos               ACCEPTED
return_reach             REFUSED   compile of /root/workspace/repo-root/probe_arms/return_reach/probe.dag produced 1 hard diagnostic(s):
variantpayload_nega      ACCEPTED
variantpayload_negb      ACCEPTED
variantpayload_pos       ACCEPTED
variantpayload_reach     REFUSED   compile of /root/workspace/repo-root/probe_arms/variantpayload_reach/probe.dag produced 1 hard diagnostic
END-TABLE
```

## What it says, read at the grammar-site grain

Thirteen arm families, folded onto the fourteen `parse_type_expr` sites of
[positions.md](positions.md). `listelem`, `mapval`, `generic` and `mapkey` are four USES of one
site (`parse_type_angle_arg`); `arg` and `paramdefault` are two uses of `parse_param`.

| site | arm | reachability control | plain kernel at a declared coproduct | arm payload at the parent |
|---|---|---|---|---|
| angle arg | `listelem` | REFUSES | **ACCEPTED** | **ACCEPTED** |
| angle arg | `mapval` | REFUSES | **ACCEPTED** | **ACCEPTED** |
| angle arg | `generic` | REFUSES | **ACCEPTED** | **ACCEPTED** |
| angle arg | `mapkey` | **ACCEPTED** | refused BY PARSE | refused BY PARSE |
| positional variant payload | `variantpayload` | REFUSES | **ACCEPTED** | **ACCEPTED** |
| callable return | `lambdaret` | REFUSES | **ACCEPTED** | **ACCEPTED** |
| callable parameter | `callableparam` | REFUSES | **ACCEPTED** | **ACCEPTED** |
| record field | `field` | REFUSES | refused | **ACCEPTED** |
| declared return | `return` | REFUSES | **ACCEPTED** | **ACCEPTED** |
| `data` initializer | `data` | REFUSES | **ACCEPTED** | **ACCEPTED** |
| fn parameter (call seam) | `arg` | REFUSES | refused | **ACCEPTED** |
| fn parameter (default value) | `paramdefault` | **ACCEPTED** | **ACCEPTED** | **ACCEPTED** |
| `let` annotation | `let` | REFUSES | **ACCEPTED** | **ACCEPTED** |
| `as` cast | `cast` | REFUSES | **ACCEPTED** | **ACCEPTED** |

**Seven sites accept a plain kernel value where a coproduct is declared** — angle argument,
positional variant payload, callable return, callable parameter, declared return, `data`
initializer, `let` annotation. That is the same seven gunbc#8925 reports, arrived at from an
independent set of fixtures, and it is stated here as a corroboration rather than a citation.
The `as` cast is an eighth accepting position; it is excluded from the seven for gunbc#8925's
stated reason — a cast is an authored assertion and whether a kernel-to-coproduct cast should
refuse is a different question.

**The arm-payload-at-parent specimen is ACCEPTED at all twelve reached positions, the record
field included.** gunbc#8876 walls that one position and is not merged, so on `main` today the
count is twelve, not eleven.

## Two cells this census adds

**A parameter's DEFAULT-VALUE expression is RESOLVED but never INFERRED.** `fn a_pd(r: Rel = 7)`
compiles clean, and so does `fn a_pd(r: Rel = nosuchname_zzz)`. It is the reachability control
PASSING that makes this a finding: at every other value-bearing position an undefined name refuses.

The structure says exactly where the gap is, and it is NOT "nothing looks at the expression" — an
earlier draft of this paragraph said that and it is false. `parse_param` parses the default and
stores it; `resolve_param` reads it back and passes it to `resolve_expr_types`. But that pass's
`ExprVar` arm returns the node unchanged with an empty diagnostic list — it resolves TYPE
references inside an expression and never binds VARIABLE references — and `04_infer` touches
`param_node_default_value` at exactly one site, the call-shape test for whether a parameter is
required. Undefined-name refusal and declared-type inhabitance both live in inference, which is
why both arms pass. Full row: gap analysis item 29.

**The map-KEY position refuses by grammar and passes by typing.** `{ 7: 1 }` and
`{ mk_inner(): 1 }` at a declared `Map<Rel, Int>` are refused as `module index refused: 1
unparseable .dag source(s)` — a parse refusal, not a type judgment — while
`{ nosuchname_zzz: 1 }` is ACCEPTED, the undefined name silently read as a string key. So the
position reads as walled from its refusal column and is not: the two refusals come from the
grammar's key form, and the one specimen that reaches typing passes.

## The pass-coverage axis, measured second

The grammar axis is not the only cut. ' `ExprVar` arm returns the node
unchanged with an empty diagnostic list, so RESOLVE can refuse an undefined name at no position
at all; inference is what refuses one. That makes "reached by resolve, not by inference" a second
axis, and it is NOT a subset of the fourteen grammar sites.

Measured, same binary:

```
BEGIN-TABLE
fielddefault_nega      ACCEPTED  exit=0
fielddefault_pos       ACCEPTED  exit=0
fielddefault_reach     ACCEPTED  exit=0
letbody_nega           ACCEPTED  exit=0
letbody_pos            ACCEPTED  exit=0
letbody_reach          REFUSED   exit=1 compile of /root/workspace/repo-root/probe_arms2/letbody_reach/probe.dag produced 1 hard diagno
paramdefault_nega      ACCEPTED  exit=0
paramdefault_pos       ACCEPTED  exit=0
paramdefault_reach     ACCEPTED  exit=0
END-TABLE
```

Two members: a PARAMETER default (`parse_param`) and a FIELD default (`parse_field`) — two uses
of two different grammar sites sharing one pass-coverage fate. The in-body `let x =
nosuchname_zzz` control REFUSES in the same run, which is what makes the two zeroes readable.

A third candidate is named and NOT counted: `resolve_transport_binding` walks a transport's
property values and children through `resolve_expr_types`, and inference touches `transport` only
to test presence. Same shape, unmeasured here — it needs a service fixture.
