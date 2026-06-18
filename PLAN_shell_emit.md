# Plan: emit shell from a modeled effect graph → ban hand-written transport shell

## Why
Recent CI PRs (#5138 the latest) smuggle unmodeled concepts in as raw shell string blobs:
`let script = "set -e\nWORKDIR=$(mktemp -d)..."`. The sniff test (operator): **a required
fallback/workaround marks an unmodeled concept.** `shell.Exec.Run` already models the *transport*
(`sh -c {script}`) but its `script: String` input is an opaque, hand-written blob — the script
*content* is unmodeled. The fix is to model the shell **program** as structured data and serialize
it, exactly as `dsl/extdeps/formats/yaml.dag` models `YamlValue` + `serialize_yaml` (#5104). Then a
lens can ban raw shell-metacharacter string literals in transport position.

## Key realization (scope reducer)
We do **not** need a new `RenderTarget::Bash` in the v1 Rust seed. `shell.Exec.Run.input.script`
is already a `String`. We model a `ShellProgram` AST + a pure `serialize_bash(p) -> String` fold in
`.dag`, and feed its output into the existing `script` field. This is a **format serializer** (peer
to the YAML one), not a new whole-language emit backend. A real `--target bash` (effect-graph →
program, §4 grammar-inverse) is the *later, fuller* version; this slice is the wedge that proves it.

## What already exists (reuse, don't re-mint — §2/§3)
- `dsl/extdeps/shell/shell.dag` — `shell.Exec.Run { input { script: String } }` (the consumer).
- `dsl/extdeps/languages/bash/syntax.dag` — `QuotingMode`, reserved words, `unquoted_metacharacters`
  (reuse for escaping/quoting decisions).
- `dsl/extdeps/languages/bash/types.dag` — `ExitStatus`, exit-code constants.
- `dsl/extdeps/formats/yaml.dag` — the **template**: `YamlValue` AST + `serialize_yaml` fold + a
  golden+discriminating witness (`dsl/test/claim/ci_yaml_serializer_witness.dag`).
- #5099 `RecordedFixture` seam — host-effect execution under `gunbc run`.

## Steps 1 → 3

### Step 1 — model the shell program as data + serialize it (the "emit shell" core)
New file `dsl/extdeps/languages/bash/program.dag`:
- `type ShellProgram = { statements: List<ShellStmt> }`
- `type ShellStmt = Command { argv: List<ShellWord> } | Assign { name, value: ShellWord }
   | Seq ... | If { test: ShellTest, then, else } | Subshell { body } | SetFlags { e: Bool, ... }
   | Raw { text: String }`  ← `Raw` is an explicit escape hatch, **marked for dissolution**, so the
   first slice can land before the AST is total; the lens (step 4, later) RED-flags `Raw`.
- `type ShellWord = Lit { text } | VarRef { name } | CmdSubst { command: ShellStmt }`
- `fn serialize_bash(p: ShellProgram) -> String` — pure fold; reuse `bash/syntax.dag` quoting facts
  for `Lit` escaping. Mirror `serialize_yaml`'s structure (per-node helpers + one top fold).

Grow the AST **incrementally**: start with `Command`/`Assign`/`Seq`/`Raw` (+ `serialize_bash`)
sufficient to render a trivial program; add `If`/`Subshell`/`SetFlags` when step 3 needs them.

### Step 2 — prove the serializer by execution (§5, not spec-without-execution)
Two witnesses (mirror the YAML serializer witness):
- **Pure golden + discriminating** (`dsl/test/claim/bash_serializer_witness.dag`): `serialize_bash`
  of a known program equals an exact string (Holds), and a perturbed program asserts a *different*
  string / RED. Runs in the claim corpus.
- **By-execution** (host): build a small `ShellProgram` (e.g. `mktemp -d` → write marker → `test`),
  `serialize_bash` it, run via `shell.Exec.Run`, assert exit 0; perturb (e.g. `test 1 -eq 2`) to
  assert RED. Runs under `gunbc run` with the host shell effect. This is the real "we can emit shell"
  proof.

### Step 3 — rewrite one real transport as the worked example
Rewrite `dsl/tools/dsl_compile_clean_transport.dag`'s `let script = "set -e\n..."` blob as a
constructed `ShellProgram` value rendered by `serialize_bash`. This forces the AST to cover:
`set -e`, `VAR=$(... )`, `if [ ! -x "$X" ]; then (cd .. && ..) || exit 1; fi`, `mktemp -d`,
command with `--flag value` args, `rm -rf`. Keep the gate's existing witness green by execution
(it already runs the compile-clean check), plus its existing perturb RED receipt. **No behavior
change** — byte-identical resulting script is the success oracle (cache-purity-style check).

## Out of scope for this plan (named, for later)
- A real `RenderTarget::Bash` / `--target bash` effect-graph→program backend (§4 grammar-inverse).
- The CI lens that RED-flags raw shell string literals / `ShellStmt::Raw` in transport position
  (the *ban* — lands only after a modeled alternative exists, i.e. after step 3).
- The other 13 `.dag` files with cemented shell (known issue) + #5138's `ci_spec.dag` retry blob.
- Modeling sccache as a Realization handler (separate concept; DESIGN.md:60 already names it).

## Risk / discipline notes
- New `program.dag` is a peer of `yaml.dag` — not a load-bearing file; low blast radius.
- No v1 Rust-seed change. No pipeline-stage change.
- `ShellStmt::Raw` lands marked with a dissolution trigger (the step-4 lens); honestly-marked
  scaffold, per DESIGN.md "every scaffold lands with a named dissolution trigger."
