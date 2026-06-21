# Plan — compile-clean gate force-check (ROADMAP §1 floor-coverage / §0 inference fail-open)

**Status:** diagnosis (done, execution-proven) + design options · **DESIGN §5 (fail-closed) is
authority.** Work-item `adhoc-6169238c-5e2`. Linked from `ROADMAP.md §0` (line 31, "inference fail-open —
return-type after #5293") and §1 (floor-coverage). Sibling of
[fail-closed-lockdown.md](fail-closed-lockdown.md) §3 (where the pipeline fails open).

**Verified against the live tree 2026-06-21** — `gunbc` built from `src/v1/stage0`, probes run. Line
numbers are receipts.

## 0. Verdict — the brief's mechanism is wrong; the leak is a global seed allowlist

The compile-clean gate (`dsl/tools/dsl_compile_clean_gate.dag` → `gunbc compile --target rust` over
`dsl/` with `src/v2` as the import pool) is **fail-open**: `gunbc compile` on `main` returns 0
diagnostics / EXIT 0 even though `dsl/extdeps/cloud/gcp/secret_manager.dag:71` calls
`utf8_decode_bytes`, which is **defined nowhere in `dsl/` or `src/v2`**.

The brief framed this as "unreached fn bodies escape typecheck." **That is false** — bodies are always
visited. The precise mechanism (execution-proven, §1 below) is two independent fail-open holes:

1. **Registry leak** — `utf8_decode_bytes` resolves because it is a hardcoded entry in the global
   `builtin_function_registry()`, an **explicitly-marked BRIDGE scaffold**. The registry is *not scoped
   to the tree being compiled*, so v1-seed runtime intrinsics leak into the dsl-substrate compile.
2. **Return-type fail-open** — a function whose body's inferred type ≠ its declared return type is not
   flagged (`#5293` closed only the record-field hole, not return types). Independent of the gate; a
   member of ROADMAP §0's "inference fail-open (return-type after #5293)".

## 1. Execution-proven mechanism (receipts)

`gunbc` built from `src/v1/stage0`; each probe is a single-file `--source-root` compile.

| Probe | Input | Result | Reading |
|---|---|---|---|
| undefined **variable**, uncalled fn | `fn f()->Int{ xyz }` | **1 error** "undefined variable 'xyz'" | bodies *are* visited; "unreached" hypothesis is false |
| call to name **not** in registry | `fn f()->Int{ totally_undefined_fn_xyz(q:3) }` | **1 error** "function '…' not found in scope" | unregistered call names already fail-closed |
| call to **registered** seed builtin | `fn f()->Int{ utf8_decode_bytes(payload:3) }` | **0 diagnostics** | registry absorbs the name → `string_type`, no def required, no arg check |
| plain return-type mismatch | `fn f()->Int{ "a string" }` | **0 diagnostics** | declared return type unenforced |

**The real on-main witness** — `dsl/extdeps/cloud/gcp/secret_manager.dag:70-72`:
```
fn utf8_secret_from_access_payload(payload: Bytes) -> Secret {
  utf8_decode_bytes(payload: payload) as Secret
}
```
`utf8_decode_bytes` resolves via the registry; the `as Secret` cast satisfies the return type. So the
**literal witness is hole #1 alone** (return-type enforcement would not catch it because of the cast).

## 2. The seam — `builtin_function_registry`

`src/v1/04_method.dag:55-155` (Rust seed `v1_compiler_infer_method.rs:100-365`). 76 names →
return-type `Node`s, e.g. `utf8_decode_bytes → string_type` (04_method.dag:93). Its own header is the
indictment:

> `// BRIDGE: This map_insert chain is a duplicate authority over facts that should come from .dag`
> `// function declarations (extern fn signatures). Deletion point: when builtins are actual .dag`
> `// definitions that the compiler loads and resolves, this registry is deleted.`

So this is a known §3 fork (duplicate authority) with a **named dissolution trigger**. Two properties
make it the fail-open:

- **Global, not tree-scoped.** The same registry is consulted whether the entry root is `src/v1` (the
  seed, which legitimately needs `utf8_decode_bytes`/`scan_while`/… as its runtime kernel — used in
  `05_emit_rust.dag`, `runtime_rust.dag`) or `dsl/` (the substrate, which must not depend on seed
  intrinsics). A name in the registry resolves in *any* tree.
- **Resolution without definition or arg-check.** A registry hit yields a fabricated return type with no
  parameter typing and no requirement that a definition exist in the compiled closure — DESIGN §5's
  "fabricated plausible output" anti-pattern, at the call-head resolution seam.

(`resolve_builtin_call_type`'s `Absent => unit_type`, 04_method.dag:166, is *not* a live leak for call
syntax — unregistered names are caught upstream as "function not found in scope". It is a latent
fail-open kept for non-call uses; out of scope here, noted for the audit.)

## 3. Construction-correct direction (DESIGN §5/§3)

The gate's claim is "the dsl substrate is well-typed **and self-contained**." Made *unwritable* (§5
construction, not a post-hoc lens): **the set of resolvable builtins must be derived from the tree being
compiled, not a global seed allowlist.** This is exactly the registry's own stated deletion point —
builtins become real `.dag` definitions resolved through normal import/definition resolution, the global
registry is deleted, and a substrate that neither defines nor imports `utf8_decode_bytes` fails closed
on it ("function not found in scope", identical to probe 2). At that point the leak is dead by
construction, not by a roster.

Full dissolution (76 builtins → `.dag` defs) is a large migration, out of this node's scope. The
bounded options below are stepping stones; (A) is the one that closes the literal witness *by
construction*.

## 4. Options (one shippable PR for this node)

- **(A) Tree-scoped builtin availability / registry partition.** Split the registry into
  *substrate-std* builtins and *seed-only kernel* intrinsics; admit the seed-only set only when the
  entry root is the v1 seed. A dsl-substrate compile then fails closed on `utf8_decode_bytes`. Closes the
  literal witness **by construction**; a stepping stone toward full dissolution. **Load-bearing**
  (inference scope) and **changes what compiles** — needs operator/parent sign-off; carries DESIGN's
  higher bar for named pipeline stages.
- **(B) Return-type enforcement.** Flag a fn whose body type ≠ declared return type. Clean §5
  construction, sanctioned by ROADMAP §0 line 31. **Does not close the literal witness** (the `as Secret`
  cast satisfies the return type). Corpus-wide enable is risky — like #5293, expect latent reds; needs a
  confident-only / staged rollout. Independent value; arguably its own node.
- **(C) Gate RED witness + ground `utf8_decode_bytes`.** Add a discriminating RED receipt to the
  compile-clean gate (plant a substrate reference to a seed-only builtin, assert compile fails) and model
  `utf8_decode_bytes` as a real `std`/`extdeps` function (RFC 3629, already cited in the gcp comment) so
  the live reference resolves within the substrate. Closes **this instance**, not the class — validation,
  not construction (§5: prefer A).

**Recommendation:** (A) is the true construction fix and the registry's own dissolution direction;
gate it on sign-off because it is load-bearing and changes compile semantics. (B) is worth doing but is a
separate fail-open class and a separate node. (C) is a fallback if a no-semantics-change PR is required
this window — but pair its gate witness with a dissolution marker so it does not masquerade as the class
fix.

## 5. Discriminating witness (must go RED when the behavior is wrong)

Whatever ships, the receipt is the same shape as probe 2/3 above: a planted substrate module that calls
a seed-only builtin (`utf8_decode_bytes`) must make `gunbc compile` over the dsl pools **fail**; the
identical module compiled with the `src/v1` seed root present must **pass**. Green-by-execution against
the real `gunbc`, not a typecheck/grep. (DESIGN §5: spec-without-execution is not done.)
