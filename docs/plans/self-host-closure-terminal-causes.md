# Reducing the v2 self-host closure's front-end refusals

The measurement this note describes is carried as data by `gunbc.tools.self_host_closure_terminal_causes`
(`dag/gunbc/instruments/self_host_closure_terminal_causes.dag`). This file carries only the METHOD —
how the table is re-derived and how each cause was reduced — because a reader who cannot re-run the
instrument has a number, not a measurement.

## The subject is the closure, not the directory

The denominator is the input closure of `src/v2/compiler/00_compile.dag`: the modules the emitted-native
compiler is actually built from. It is derived, not listed — the emitted probe crate writes one Rust file
per closure module, each carrying a `// Source module:` header, and joining those headers against every
`.dag` file's own `module` line yields the closure manifest. A directory census under `src/v2/compiler`
is a different and much smaller population and is not the subject.

## Step 1 — the population and its terminal causes

```
claim_executor --required-ci --source-root dag --source-root src/v2 --required-lane v2-native
```

The lane emits the closure, cargo-builds it, and runs the emitted `SourceRootEvalDriver`, whose context
fold prints one `{"file_refusal": {path, head_reason, fatal_reason}}` row per source it refused. The
`fatal_reason` is the terminal cause; the `head_reason` is the grammar-global overlap residue that
`rejected_with_pending` prepends to every parse failure and discriminates nothing.

Two environment facts that cost time if unknown:

- The lane resolves `cargo` through `$CARGO` or `PATH`. A session whose `PATH` puts a build-wrapper shim
  first, or whose `RUSTC_WRAPPER` points at an sccache that cannot open its socket, produces
  `EmittedCompilerBuildFailed — Completed status=101` with the real stderr swallowed by
  `cargo_verdict_summary`. Reproduce `run_cargo`'s exact invocation by hand before concluding anything
  about the emitted Rust.
- The driver takes `<universe.tsv> <source-root>...`, and the per-file refusal rows come from its
  whole-ingest context fold, not from the universe. A controls-only universe file is therefore enough to
  collect them, which avoids paying the full `v2.test.*` run for each observation.

The closure is import-closed, so a source root holding exactly the closure files yields identical
per-file rows in a fraction of the time; a per-file front-end refusal is a function of that file's bytes
and the prepared grammar alone.

## Step 2 — the reduction probe

The stop lexeme a parse refusal reports is where the parser gave up, not the cause; the cause usually
lives inside the declaration BEFORE it. Reducing therefore needs a per-file probe that reports the whole
ordered diagnostic list, through the same path the driver's fold takes so that every stage — tokenize,
parse, normalize, namespace graft, body lowering — is reachable. Add this as `src/bin/reduce.rs` in the
emitted probe crate and build it with `cargo build --release --bin reduce`; it costs about 0.3 s per
file and amortises the grammar preparation across a batch:

```rust
// Reduction probe: one file in, its full ordered diagnostic list out, through the SAME
// per-file front-end path the native driver's context fold takes.
#![allow(clippy::all)]
use std::rc::Rc;

use v1_compiled::extdeps_communication_medium::{DecodeFidelity, Medium};
use v1_compiled::v2_compiler_program_assembly::program_assembly_read_to_normalized_root_prepared;
use v1_compiled::v2_compiler_parse::prepare_grammar;
use v1_compiled::v2_compiler_source_authority::{source_root_for_storage_path, DagSourceReadWitness};
use v1_compiled::v2_std_artifact::{Artifact, ArtifactKind};
use v1_compiled::v2_std_diagnostic::Outcome;

fn main() {
    let paths: Vec<String> = std::env::args().skip(1).collect();
    if paths.is_empty() {
        eprintln!("usage: reduce <file.dag>...");
        std::process::exit(2);
    }
    let lm = v1_compiled::v2_extdeps_languages_dag::dag_language_model();
    let (prepared, residue) = match &*prepare_grammar(lm.grammar.clone()) {
        Outcome::Rejected { diagnostics } => {
            println!("{}", serde_json::json!({"grammar_prepare_refused": diagnostics}));
            std::process::exit(1);
        }
        Outcome::Accepted { value, diagnostics } => (value.clone(), diagnostics.clone()),
    };
    for path in &paths {
        let text = match std::fs::read_to_string(path) {
            Ok(t) => t,
            Err(cause) => {
                println!("{}", serde_json::json!({"path": path, "read_failed": cause.to_string()}));
                continue;
            }
        };
        let read = Rc::new(DagSourceReadWitness {
            source: Rc::new(Medium {
                carried: text,
                fidelity: DecodeFidelity::Lossless,
                _phantom: std::marker::PhantomData,
            }),
            artifact: Rc::new(Artifact {
                kind: ArtifactKind::SourceFile,
                id: path.clone(),
                file_path: path.clone(),
            }),
            compilation_unit: path.clone(),
            source_root: source_root_for_storage_path(path.clone()),
        });
        let outcome = program_assembly_read_to_normalized_root_prepared(
            read,
            lm.clone(),
            prepared.clone(),
            (*residue).clone(),
        );
        match &*outcome {
            Outcome::Accepted { .. } => {
                println!("{}", serde_json::json!({"path": path, "verdict": "accepted"}));
            }
            Outcome::Rejected { diagnostics } => {
                println!(
                    "{}",
                    serde_json::json!({"path": path, "verdict": "rejected", "diagnostics": diagnostics})
                );
            }
        }
    }
}
```

Its calibration is that it reproduces the driver's own file-refusal rows cause-for-cause.

## Step 3 — the three reduction moves, in the order they pay

1. **Isolate the declaration.** Split the file at module-scope declaration boundaries (`fn`, `test fn`,
   `data`, `type`, `test`, `service`, `func`, `module`, `import`), rebuild each non-import declaration
   beside the module header and the imports, and probe them as one batch. The declarations whose
   refusal reason equals the file's terminal reason are the culprits. This is much cheaper than prefix
   bisection and it finds every culprit rather than the first.
2. **Climb a ladder.** From a known-good base, make one well-formed edit at a time until the verdict
   flips. A cause is not established until it has a rejecting reproducer, a positive control that
   accepts, and a control that separates it from its nearest alternative — the third is what stops a
   confound riding along. Two worked examples: an em dash was the only non-ASCII character shared by
   the lexer class's refusing declarations and is innocent (em dash, en dash, accented letters, arrows
   and raw tabs in strings all accept); the real cause is a raw newline inside a string literal. And a
   keyword used as a name is fine in declaration position and refuses only in expression position, so a
   control on the wrong side of that boundary would have named the wrong subject.
3. **Delta-debug what the ladder cannot guess.** Remove chunks of the declaration while the terminal
   reason holds. Two guards are mandatory, and both were learned by getting a wrong answer first: the
   candidate must be brace-balanced AFTER string literals are blanked (otherwise a `"{"` inside a string
   lets an unbalanced candidate through), and the minimal result must be checked against the ladder,
   because delta-debugging can slide into a DIFFERENT defect that happens to report the same reason —
   it reduced one file to an empty `if` block, which refuses for its own unrelated reason.

## Step 4 — joining causes to files, and saying which join you have

Attributing a cause to a file by matching its construct in the isolated declaration is a signature match.
Where the construct can be removed mechanically — a `where` clause, `sole_constructor`, a leading pipe, a
numeric type argument, a negative literal, a `return`, an expression body — the join can be EXECUTED
instead: neutralise the construct in the isolated declaration and re-probe it. A declaration that still
refuses after neutralisation carries a second cause as well, which is a co-occurrence and not a
counterexample. The carrier records which of the two backings each cause has, per `JoinEvidence`, because
calling a signature match executed would be rung inflation.
