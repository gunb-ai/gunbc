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

## What is an artifact here, and what is only a recipe

Two different things are described below and they have different standings, so the distinction is
stated before either.

**The route that produced the table is a required CI lane — and it resolves at the PINNED HEAD, not on
main.** The `v2-native` lane, its authority `gunbc.witness_v2_native_route`, and the
`NativeTestFileRefusal` row type are #10882's and are unmerged at this writing; on any base without that
PR the three names resolve to nothing. The population and the terminal causes in
`gunbc.tools.self_host_closure_terminal_causes` come from there and nowhere else, so the pin's
`head_sha` is the tree a reader must check out to re-derive them. Saying "in this repository" without
that qualification, as an earlier revision of this note did, is the unreachable citation the carrier
refuses to make about its own probe digest.

**The per-declaration reduction probes below are RECIPES, not artifacts.** They are hand-written Rust
that lives in the emitted probe crate for the length of a reduction session and is not committed:
`src/bin/reduce.rs` is not a tracked path, and the carrier deliberately does not pin a digest of it,
because a digest of something no reader here can rebuild is an unreachable citation and worse than
none. Their **dissolution condition**: they die when the model can emit a driver that reports per-file
front-end diagnostics — the same move `SourceRootEvalDriver` already makes for per-declaration
verdicts, which is a `CompilerEntryDriver` change and therefore a compiler PR, not a measurement one.
Until that exists, treat what follows as the method a reader re-executes, not as a component.

## The order-insensitive closure digest, and its exact producer

The emitted closure's reported identity is a digest over bytes, and the seed emitter writes each
module's `pub use crate::…` re-export lines in unordered-set order, so two emissions of the same head
report different identities. The carrier therefore pins an ORDER-INSENSITIVE digest beside it as the key
two runs can join on. This is that field's producer, and it is written out in full because sixteen
plausible readings of a prose description of it all produced different values on another host:

```
(cd src && for f in $(find . -type f -not -path './bin/*' | LC_ALL=C sort); do
   echo "== $f"
   grep -v '^pub use crate::' "$f" | LC_ALL=C sort
 done) | sha256sum
```

run from the emitted crate directory. Every detail is load-bearing: the `== ./path` header line per
file is inside the hash; the file list is `find`-order sorted under `LC_ALL=C`, not shell glob order;
the stripped prefix is exactly `pub use crate::` at line start, not `use ` and not any `pub use`; each
file's surviving lines are sorted under `LC_ALL=C`; and the whole stream is hashed once rather than per
file. Expect 171 files. A different count means the two emissions differ and no row join between them
is valid.

**`-not -path './bin/*'` is not incidental.** The reduction probes below are written INTO the emitted
crate's `src/bin/`, and an earlier revision of this note's digest swept them in — making the pinned
value a fact about whoever had probed rather than about the emission, and unreproducible by anyone
else. That is the same unreachable-citation defect this note warns about for the probe binary itself.
The digest must cover the emitted sources and nothing else.

## Step 2 — the reduction probe

The stop lexeme a parse refusal reports is where the parser gave up, not the cause; the cause usually
lives inside the declaration BEFORE it. Reducing therefore needs a per-file probe that reports the whole
ordered diagnostic list, through the same path the driver's fold takes so that every stage — tokenize,
parse, normalize, namespace graft, body lowering — is reachable. Reconstruct it as `src/bin/reduce.rs` in the
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

## The grammar-preparation residue, and how to report movement on it

`prepare_grammar` carries a non-fatal residue, and it is that residue `rejected_with_pending` prepends
to every parse failure — which is precisely why the head grain discriminates nothing. A repair lane
adding a production needs it as a BEFORE number, so the carrier pins THE READING - which is a fact
about the grammar at a named head, and reproducible from the tree through the lane - while the probe
that took it stays a recipe of the same standing as `reduce.rs`, `src/bin/residue.rs`:

```rust
// The grammar-preparation residue at a pinned head: how many non-fatal diagnostics
// prepare_grammar carries, and their reason histogram. This is the advisory that
// rejected_with_pending prepends to every parse failure.
#![allow(clippy::all)]
use std::collections::BTreeMap;
use v1_compiled::v2_compiler_parse::prepare_grammar;
use v1_compiled::v2_std_diagnostic::Outcome;

fn main() {
    let lm = v1_compiled::v2_extdeps_languages_dag::dag_language_model();
    match &*prepare_grammar(lm.grammar.clone()) {
        Outcome::Rejected { diagnostics } => {
            println!("{}", serde_json::json!({"grammar_prepare": "rejected", "diagnostics": diagnostics}));
        }
        Outcome::Accepted { diagnostics, .. } => {
            let mut hist: BTreeMap<String, usize> = BTreeMap::new();
            let mut total = 0usize;
            if let Some(ne) = diagnostics.as_ref() {
                total += 1;
                *hist.entry(ne.head.reason.to_string()).or_default() += 1;
                for d in ne.tail.iter() {
                    total += 1;
                    *hist.entry(d.reason.to_string()).or_default() += 1;
                }
            }
            println!("{}", serde_json::json!({"grammar_prepare": "accepted", "residue_total": total, "residue_by_reason": hist}));
        }
    }
}
```

It is a property of the GRAMMAR, not of the corpus. A lane adding a production can move it in either
direction, and a rise is not by itself a regression — a production that overlaps an existing choice
adds residue rows even when it parses everything it was added for. The honest report is the new total
together with which choices its rows name, never "residue unchanged".

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

**A signature match must never see annotation text, and this is not hypothetical.** The first pass
matched the bare word `where` against the whole declaration, and on `dag/std/measure.dag` it matched the
English word inside a `//` block — attributing a `where`-clause defect to a file that carries none. The
repair lane taking that row would have hunted for a clause that was not there. Strip every line whose
first non-space characters are `//` before matching, and re-check the population afterwards: over the
26-file parse class exactly one attribution changed, which is the shape of a defect this cheap to make
and this quiet to miss. It is also the sharpest argument for the `JoinEvidence` split — the executed
neutraliser join cannot make this mistake, because it re-runs the compiler on the modified declaration
and watches the refusal go away.

Attributing a cause to a file by matching its construct in the isolated declaration is a signature match.
Where the construct can be removed mechanically — a `where` clause, `sole_constructor`, a leading pipe, a
numeric type argument, a negative literal, a `return`, an expression body — the join can be EXECUTED
instead: neutralise the construct in the isolated declaration and re-probe it. A declaration that still
refuses after neutralisation carries a second cause as well, which is a co-occurrence and not a
counterexample. The carrier records which of the two backings each cause has, per `JoinEvidence`, because
calling a signature match executed would be rung inflation.
