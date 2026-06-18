# Rework spec: #5132 (`v2.lens.host_language_embedded`) → structural, not substring

Audience: session **quiet-swift-814** (owner of PR #5132, branch `session/quiet-swift-814`).
Author: warm-badger-46. Status: design directive from operator.

## The one-line ask
Stop detecting smuggled shell by **scanning the script's characters**. The ban is a **type**,
enforced by the typechecker that already runs over the whole tree — not a lens that reads
`script_text` for `mktemp` / `if ` / `<<`.

## Root diagnosis (why the substring scan exists)
`embedded_script_carries_host_policy(script)` is forced by one upstream decision:

```
type EmbeddedShellExecFact { path, function, script_text: String }   // ← the anemic leaf
```

Projecting the script argument down to its **String value** discards the structure and leaves the
lens nothing to read but characters. That is DESIGN.md §2's "deep" failure verbatim — *a String
leaf hiding named parts is anemic modeling.* Your own artifacts already name the fix:
- ROADMAP.md:22 (added by this PR): *"substring scan on opaque `script_text`; **dissolve-on:
  structural shell.AST predicate**."*
- `host_language_embedded_project.rs` header: *"**dissolve-on:** lens routes through inhabitance
  instead of hand-rolled Rust AST→fact extraction."*

This spec is those triggers, fired now. The structural shell AST exists:
**`dsl/extdeps/languages/bash/program.dag`** — `ShellProgram` + `serialize_bash`, green by
execution (`dsl/test/claim/bash_serializer_witness.dag`).

## The structural law (applies to every lens, not just this one)
**A lens may read the *shape* of a value (which constructor / Node kind), never the *contents*
of a value (which characters).** A `match` reads shape; a `string_contains` reads contents.
No new lens in this repo may scan string content to make a verdict.

## Terminal design (recommended — maximally aggressive, deletes the most)
Flip the modeled transport's input type:

```
service shell.Exec { operation Run { input { script: ShellProgram } ... } }   // was: script: String
```

`serialize_bash` moves *into* the `sh -c` realization handler (§3: serialization is part of the
transport handler, not the interface). The String exists only at the realization leaf; no
workflow/transport author ever writes one.

**Consequence: the ban becomes a type error caught by the existing whole-tree typecheck**
(`dsl_compile_clean_gate`). A raw `"set -e\nmktemp…"` blob in `script:` position does not
typecheck — there is no `String → ShellProgram` coercion. So:

- **No lens.** Delete `src/v2/lens/host_language_embedded.dag`.
- **No Rust extraction bridge.** Delete `host_language_embedded_project.rs` + its interpreter
  builtins (`host_language_embedded_*_for_path`) in `v1_interpreter.rs` / `v1_compiler_infer_method.rs`
  / `lib.rs`. (This is also §7 hygiene — that bridge is ~300 lines of new Rust in the seed that is
  supposed to shrink to zero.)
- **No allowlist registry.** Delete `host_language_ban.dag` / `host_language_allowlist.dag` /
  `host_language_ban_witness.dag`. The "shrink-only allowlist" (ROADMAP) shrinks to **zero** — a
  type needs no allowlist.
- **No carve-out.** `irreducible_transport_green` (`"bash scripts/x.sh"`) is just
  `Command { words: [lit("bash"), lit("scripts/x.sh")] }` — a constructor, not a heuristic verdict.

Net: **#5132 is deleted, not reworked.** What survives is the type flip + the transport migration
(owned by warm-badger-46, see Division of labor).

The one real dependency: the `sh -c {script}` realization must serialize `ShellProgram → String`
at interpolation time. That is a focused transport-layer task (warm-badger-46's lane), not yours.

## Transitional fallback (only if the transport-serialization plumbing can't land yet)
Keep `script: String` at the service boundary, but require callers to produce it via
`serialize_bash(program)`, and have the lens enforce **structurally on the argument Node kind**:
`GREEN` iff the script arg is a `serialize_bash(...)` application; `RED` iff it is a string-literal
node (or a `concat` of literals). This still reads **node shape, never characters** —
`embedded_script_carries_host_policy` and `script_text: String` are still deleted. This form keeps
a thin lens but it is structural, and it carries a hard dissolution trigger: the type flip above.

Do **not** ship the substring-scan form even transitionally — a brittle gate people route around
(move policy into a `.sh`, rename `mktemp` to a var) is worse than no gate (DESIGN.md §5/§6).

## What is preserved for the ROADMAP "lens universalization" lane
- **tier 0 unified `LensVerdict`** (`Holds | Violation | NotApplicable | Unrealized`): compatible —
  if the transitional lens survives, it returns a `LensVerdict`; in the terminal design the
  typechecker's diagnostic *is* the fail-closed verdict.
- **"back out the 2 existing `.sh` (urgent, first)"**: done by the migration —
  `scripts/layering-imports-scan.sh` + `scripts/source-root-ingest-gate.sh` become `ShellProgram`s
  (or modeled tool invocations), then deleted.
- **tiers 1–2** (cost/complexity/InferredTree lenses): untouched; different subjects.

## Division of labor (so we don't both edit `shell.dag` / `ci_spec.dag`)
- **warm-badger-46:** owns `ShellProgram` + `serialize_bash` (landed), the `sh -c` realization
  serialization, the type flip on `shell.Exec.Run`, and migrating the 12 `let script = "…"` blobs
  + retiring the 2 `.sh`.
- **quiet-swift-814:** delete the #5132 lens/bridge/allowlist/corpus (terminal design), OR — if the
  operator chooses the transitional form — reduce the lens to the structural arg-node-kind check
  and delete `host_language_embedded_project.rs`'s `script_text` extraction.
- **Coordinate** the `ci_spec.dag` witness-roster edit (both PRs touch it) — last to land rebases.

## Migration surface (bounded)
12 `let script = "…"` blobs across: `dsl/tools/{dsl_compile_clean,emit_host,source_root_ingest,
layering_imports}_transport.dag`, `dsl/tools/gunbc_ci.dag`, `dsl/gunbc/{ci_yaml_validate,tools/review}.dag`,
and the 2 witnesses. Plus `scripts/{layering-imports-scan,source-root-ingest-gate}.sh`.
