# SHELL-DAG-CENSUS-0A — projection blocker and derived shell-interpretation seed

Status: finding. Produced under the stop condition in
[shell-dag-census-0a-brief.md](shell-dag-census-0a-brief.md), which reads:

> If the available parsed-body facts cannot preserve exact callee identity, argument edges,
> branch identity, or interprocedural value flow, stop. File the missing typed projection as a
> separate substrate increment. Do not substitute text scanning.

Subject: `a6ca6882d18114b52532c0804dc89f97b441f493` (first `main` SHA satisfying the brief's
precondition — #8652 merged, `witnesses` workflow SUCCESS on that exact SHA, run 32370279129).

No detector was built. No text-scanning census was substituted. What follows is (1) why the
available fact surface cannot carry 0A, axis by axis, each with its own remedy, and (2) the
shell-interpretation seed derived from what actually interprets bytes as shell, which is needed
under whichever remedy lands.

---

## 1. The whole fact surface `.dag` can read

`src/v1/stage0/src/coproduct_reflection.rs` exposes exactly three live-fact accessors:

| accessor | item filter | `.dag` surface | payload |
| --- | --- | --- | --- |
| `eval_concept_decl_facts_live` | `ItemKind::TypeItem` | `v2.std.concept_index` `ConceptDecl` | `qualified_name`, `name`, `node` |
| `eval_fn_arrow_decl_facts_live` | `ItemKind::FnItem \| ItemKind::FuncItem` | `v2.std.fn_index` `FnArrowDecl` | `qualified_name`, `name`, `output`, `params` |
| `eval_data_init_decl_facts_live` | `ItemKind::DataItem` | `v2.std.data_index` `DataInitDecl` | `qualified_name`, `module`, `name`, `literal_fp` |

That is the entire typed body-fact projection. `ItemKind::ServiceItem` exists in the seed's item
vocabulary and has **no accessor**.

The one prior art in this space, `v2.lens.effect_reach`, reads that surface and carries a
`ShellExecRunSink` variant — but `sink_kind_for_callee` decides by `string_eq` against a
remembered callee-text list (`"Run"`, `"shell.Exec.Run"`, `"Exec.Run"`), everything else falling
to `UnknownHostEffectSink`. The brief's disposition on it is *supersede, not extend*, and the
reason is visible in the table above rather than in that function: the surface it reads cannot
express the alternative.

## 2. Six axes, six remedies

### Axis 1 — service operations and transports are absent entirely

The fact that decides what interprets bytes as shell lives in transport declarations:

```dag
service shell.Exec {
  operation Run {
    input { script: TransportScript }
    transport shell { argv: ["bash", "-s"] stdin: script.body }
  }
}
```

`ItemKind::ServiceItem` has no accessor, so no `.dag` consumer can read any of it — not the
operation, not the argv, not the stdin channel. The brief permits exactly one seed ("what
INTERPRETS bytes as shell") and forbids the alternatives (a file list, a call-site roster, a
variant-name list). In this tree the permitted seed is *only* obtainable from these declarations.
A census over the available surface cannot have a permitted seed at all.

**Remedy:** a typed service/operation/transport declaration projection.

### Axis 2 — `FnArrowDecl.output` is a wiring-liveness skeleton, not a body

`marshal_stmt_sequence` folds statements right-to-left carrying a live-reference set back from
the return, and grafts a `let b = rhs` **only** when `b` is referenced downstream of the binding
by code that itself reaches the return. Its own comment states the intent: a parameter used only
inside a dead `let` must be absent so it flags as a dead wire. Correct for wiring liveness;
fatal here. A shell call whose result is bound and not consumed is **absent from the input**, so
`NoStaticShellRoute` is returned for a body that contains a route.

**Proven by execution, not by reading the marshal.** A fixture with two functions of identical
shape — one binding the call and not using it, one returning it — folded through
`fn_arrow_decl_facts_live` and the atom identities collected from each `output`:

```text
fn fixture_dead_let_shell() -> String {
  let unused_result = fixture_sink(program: "echo DEADLETMARKER")
  "returned-without-using-unused_result"
}
fn fixture_live_named_args() -> String {
  fixture_sink2(program: "echo LIVEMARKER", args: "echo ARGSMARKER")
}

DECL probe.fixture.fixture_dead_let_shell   ATOMS: (empty)
DECL probe.fixture.fixture_live_named_args  ATOMS: fixture_sink2 | echo LIVEMARKER | echo ARGSMARKER
```

The dead-let arm yields **nothing**: the callee identity and the program literal are both absent.
The live arm yields both. That is the discriminating pair — same construct, opposite verdict —
so the loss is the projection's, not the probe's.

This is a structural false zero upstream of the detector. No instrument control in the brief's
seven can catch it, because every one of them is a control over the detector, and the loss has
already happened in the detector's input.

**Remedy:** a body projection whose fold is total over statements — effect positions retained
independently of return reachability.

### Axis 3 — no named-argument edges

`marshal_generic` emits positional child edges plus string-literal atoms, and
`hoist_call_arg_string_literal_edges` lifts literals from arbitrary depth up to the call node,
discarding which argument they came from. Argument labels never appear. The brief requires "the
exact program-channel argument edge", and §4 of the seed derivation below shows why: in
`["sh", "-c", "command -v \"$1\"", "sh", "{command}"]` the program is the *third* element and
`{command}` is data, while in `["sh", "-c", "{command}"]` the same-named value *is* the program.
Positionally-erased arguments cannot distinguish those.

The same executed run shows it directly. `fixture_sink2(program: "echo LIVEMARKER", args: "echo
ARGSMARKER")` projects to `fixture_sink2 | echo LIVEMARKER | echo ARGSMARKER` — three atoms in
authored order with **no labels**. Nothing in that output says which literal was `program:` and
which was `args:`, and the ordering is an accident of authoring rather than a carried fact.

**Remedy:** argument edges labelled with the authored argument name, and positional index
preserved, at each call.

### Axis 4 — no arm or occurrence identity

Match arms are undifferentiated positional children. Nothing in the surface can construct the
brief's `BodyArmIdentity` or `OccurrenceId`. The consequence is not only a missing field: the
historical calibration is stated at arm grain (36 production arms across 14 modules, 1
realizing), so the calibration is not reproducible against this surface at all — the detector
could not disagree with it or agree with it.

**Remedy:** stable occurrence identity on body nodes, and arm identity on match arms.

### Axis 5 — callee is an authored lexeme, not a resolved identity

Callee atoms, string literals, record-construction spellings and variant names all arrive as the
same `atom_identity_node` kind. The brief's exact-identity control has two halves. The easy half
passes: `words` and `keywords` are different atoms. The hard half fails: `"ShellCommand"` as a
string literal and `ShellCommand` as a constructor are the same atom, so a prose or literal
mention is indistinguishable from a use. That is the false-positive direction of the same control.

**Remedy:** resolved declaration identity on callees, and a node-kind discriminator separating
literal, callee, constructor and variant occurrences.

### Axis 6 — the registry is the entry's import closure, not the corpus

Measured rather than read: a probe folding `fn_arrow_decl_facts_live()` under
`--source-root dag --source-root src/v2` reported **1,698** fn/func declarations. The corpus
declares **41,965**. The surface is the *entry's loaded-module closure* — about 4% of the tree,
and its content is whatever that entry happened to import.

This is a declared frontier rather than a discovery. `v2.lens.affected_set.corpus_dependency_view`
already guards it: `corpus_dependency_view_per_pr_substrate_ready` calls
`fn_arrow_decl_substrate_is_whole_tree`, and when it is false
`ensure_corpus_dependency_view_per_pr_substrate` routes to a host refusal whose message reads
`corpus_dependency_view per-PR execution refused ... (blocked-on-#6239)`. Fail-closed, correctly.
A census over this surface inherits that refusal.

It is fatal for 0A specifically because the brief's corpus requirement is a denominator claim —
"Scan all authored `.dag` files under the declared production roots ... No file may disappear
through exclusion" — and here files disappear through *non-import*, silently, with no per-file
`ParseRefused` row to count them. That is the empty-observation narrow named in DESIGN's failure
modes: the population that was never loaded is indistinguishable from the population that carries
no route.

**Remedy:** a corpus-grain producer whose denominator is an enumerated file set, not an import
closure.

## 3. The worked proof — `gunbc.spark_managed_access_apply`

The brief names four script producers reaching a host through `stdin_payload` behind
`sudo -S sh -s`, and requires that a correct detector find them **without being told their
names**. That chain is the proof that the five axes above are a capability gap and not an API
preference, because each hop is one of them.

In `privileged_leg`:

1. `portable_remote_words(raws: ["sudo", "-S", "sh", "-s"])` — the interpreter words are a list
   literal in a **named argument** (axis 3).
2. `match … { Present { value: words } => … }` — the words are bound by a **match-arm binder**
   (axis 4; and the binder is not a fn parameter, so it is not even emitted as a
   parameter-reference atom).
3. `shape_fleet_ssh_exec(…, words: words)` then `ssh_session_exec_portable_words_with_stdin(client_args: …, stdin_payload: …)`
   — interprocedural flow through **named arguments** into a **service operation** (axes 1, 3, 5).
4. `ssh.Session.ExecPortableWordsWithStdin` declares `transport shell { argv: ["ssh", client_args] stdin: stdin_payload }`
   — the shell program travels on the **transport stdin channel** (axis 1).

There is no hop in that chain the available surface can see. A detector over it finds these four
producers only by being told their names, which is the defect this cut exists to end.

## 4. The derived shell-interpretation seed

Independent of which remedy lands, the census needs a vocabulary for what interprets bytes as
shell, derived rather than remembered. The important finding is that the discriminator is
**positional, not a program-name list**, and that the shell-interpretation locus is **not only
the `extdeps` declaration layer**.

Corpus-wide (`dag/`, `src/v2/`) there are **262** `transport shell` declarations. The
overwhelming majority are **direct execs** whose transport is merely *named* shell — `find`,
`test`, `mv`, `chmod`, `uname`, `mktemp`, `systemctl`, `tmux`, `xorriso`, `jq`, `wc`, `whoami`,
`true`. No byte of their argv is parsed as a shell program.

**17** sites in the tree carry an argv literal whose word list names a shell interpreter, and
they partition by *locus*, which is the load-bearing part:

| locus | count | files |
| --- | --- | --- |
| `extdeps` `transport shell` declarations | 6 | `extdeps/shell/exec.dag` (2), `extdeps/shell.dag`, `extdeps/ssh/session.dag`, `extdeps/entropy/entropy.dag` (2) |
| product **fn-body `ArgvCommand` record constructions** | 10 | `gunbc/fleet_probe_identity_observe.dag` (6), `gunbc/fleet_host_key_enrollment.dag` (4) |
| test | 1 | `test/manual/command_runner_local_argv_receipt_test.dag` |

The 10 product-body sites matter more than the 6 declarations: they are `ArgvCommand { argv:
["sh", "-c", …] }` records built **inside fn bodies**, one of them with a *computed* program
(`argv: ["sh", "-c", pin_cmd]` in `fleet_host_key_enrollment`). A projection that covered only
service declarations — axis 1 alone — would find the 6 and miss the 10, including the only
computed-program site in the tree. Both the declaration seed and real bodies are required.

Four structural rules fall out, and none of them is a name list:

1. **The interpreter is at the program position, and some wrappers re-root that position — but
   `--` does not tell you which.** 46 argv literals contain a `"--"` element. 41 are
   `ssh`-prefixed, where `--` genuinely re-roots: the words after it are a fresh argv executed on
   the far host, which is why `["ssh", "{host}", "bash", "-s"]` is a shell route and
   `["sshpass", …, "--", "test", "-w", "{path}"]` is not. The other 5 are not one class:
   `["systemd-run", "--unit={unit}", "--collect", "--", command_argv]` re-roots into a new program
   (not a shell); `["cargo", "run", "-p", package, "--bin", bin, "--", args]` passes *arguments* to
   an already-named program; `["git", "-C", "{repo}", "ls-files", "-u", "--", "{path}"]` is plain
   POSIX end-of-options and re-roots nothing.

   So `--` is overloaded three ways and no syntactic rule separates them. Which reading holds is a
   fact about the *host program's* CLI contract — `ssh(1)`, `systemd-run(1)`, `cargo`, `git(1)` —
   which is an `extdeps` citation duty (DESIGN §3), not something the census may infer from the
   token. The census must therefore consult a per-program re-root contract, and where no such
   contract is modeled the honest answer is `StaticShellRouteUnknown`. A census that treated `--`
   as "re-root" uniformly would read `git ls-files -- {path}` as executing `{path}`; one that
   treated it as "end of options" uniformly would miss every remote route. This is the single
   place in the seed where a *new modeled authority* is owed rather than a derivation.

2. **The flag selects the channel.** `-c`/`-lc` puts the program in the *next argv element*; `-s`
   puts it on **stdin**. These are different edges, and a census that reads only argv misses
   every `-s` route — which includes `shell.Exec.Run`, the tree's main script route.
3. **A program-position value is not necessarily a program.** `shell.PosixCommand.Check` declares
   `argv: ["sh", "-c", "command -v \"$1\"", "sh", "{command}"]`: the script is a *fixed literal*
   owned by the declaration and the caller's value is bound to the shell's positional `$1`, so it
   never enters the text parsed as shell. Reading `{command}` as a program source is a false
   positive. Two of the `entropy.dag` routes are the same shape — fixed literal scripts with an
   interpolated `head -c {count}` length. The discriminator is which argv slot a value occupies,
   which is why axis 3 (argument edges) is not a nicety.
4. **Some operations are undecidable at their declaration.** `ssh.Session.ExecPortableWords` and
   `ExecPortableWordsWithStdin` declare `argv: ["ssh", client_args]` — the program position is
   *inside* an opaque expansion. Whether they interpret shell depends entirely on what a caller
   flows into `client_args`; the spark caller flows `["sudo", "-S", "sh", "-s"]` and makes it a
   shell route. At the declaration these are `StaticShellRouteUnknown`, never
   `NoStaticShellRoute`.

Rule 4 is the load-bearing one for the increment: it proves the seed alone is insufficient and
that the census is necessarily *declaration seed joined with interprocedural value flow*. Neither
half is optional, and the value-flow half is exactly what axes 2–5 remove.

A fourth locus exists and is called out so it is not mistaken for absence: three test fixtures
carry `service … transport shell { argv: [\"sh\", \"-lc\", \"{script}\"] }` inside **string
literals** of `.dag` source (`pipeline_transport_emit_rest_shell_witness_test.dag`,
`transport_script_wall_compile_red_test.dag`). Those are program text *about* shell transports,
not declarations, and they belong in the brief's Test partition. They are also the exact
false-positive shape axis 5 cannot discriminate.

## 5. The full-fidelity route: measured, and it is a third outcome

The full-fidelity route does exist in `.dag`: `v2.compiler.source_authority`
`parse_dag_source_ast` takes a `Medium<String>` to `Outcome<DagSourceAst>` where
`ParseTree = Node`, with `normalize`/`resolve` above it giving `ResolvedTree = Node`; and
`Filesystem.List`/`Filesystem.Read` are callable from `.dag`
(`v2.workflow.floor_discovery_producer` already walks scan dirs with both).

But `parse_dag_source_ast` has **no consumer**. Its only two call sites are inside
`canonical_dag_source_parse_print_law`, and that function's name occurs exactly once in the
entire tree — its own definition. It has no callers at all. It is a parse/print law that nothing
executes: DESIGN §5's specification-without-execution class, sitting in the compiler's own source
authority. So "the route exists in `.dag`" was not evidence that it works, and it had to be
measured. That specimen is worth a row wherever the enforcement-intent registry ends up,
independent of this census.

### Correctness: one named grammar class refuses a large fraction of the corpus

Three arms in one run, the first a positive control so a refusal cannot be blamed on the harness:

```text
CONTROL_3LINE (the tree's own "module m / fn add" fixture)   => ACCEPTED
REAL_SMALL    (dag/gunbc/bash_materialized_transport.dag)    => ACCEPTED
REAL          (dag/extdeps/shell/exec.dag)                   => REJECTED
                                                reason = parse_grammar_choice_overlap_residue
```

The route is real — it parses authored corpus source, not just fixtures — and its failure is a
**located, typed refusal naming a specific grammar deficiency**, which is the fail-closed
`ParseRefused { path, cause }` shape the brief anticipates.

`extdeps/shell/exec.dag` is not a corner case. On a random 10-file sample of the real corpus:

```text
A  src/v2/std/constraint_satisfaction_predicate.dag
R  dag/test/claim/filesystem_read_hermetic_witness.dag                    parse_grammar_choice_overlap_residue
R  dag/test/claim/wet_hermetic_equivalence_witness_test.dag               parse_grammar_choice_overlap_residue
R  src/v2/test/claim/.../cargo_fmt_dead_param_test.dag                    parse_grammar_choice_overlap_residue
A  dag/test/fixture/sole_constructor_sealed/admitted_caller.dag
A  dag/extdeps/transports/file.dag
A  src/v2/workflow/host_discovered_owned_data_manifest.dag
A  src/v2/lens/structural_similarity.dag
R  src/v2/test/claim/round_trip/source_authority_contract_test.dag        parse_grammar_choice_overlap_residue
A  dag/extdeps/cache/catalog_placement.dag

6 accepted, 4 refused
```

The grouping is the finding, not the rate: **all four refusals, and the `shell/exec.dag` refusal,
carry the same reason** — `parse_grammar_choice_overlap_residue`. This is one grammar deficiency
with many victims, not a scattered set of file-specific problems. That matters in both directions:
the route is a single repair away from a much larger accepted population, and no census can run on
it until that repair lands, because the brief's merge bar is *zero* production parse refusals and
the refusal set includes the census's own seed file.

### Affordability: measured, and it is the third outcome

Correctness alone would send a detector out that cannot run, so accepts-or-refuses was the wrong
frame. There are three outcomes: refuses (substrate cut); accepts and is affordable at corpus
grain (projection lands first, census rebases); and **accepts but is unaffordable at corpus
grain**, which is neither. Measured on a random 40-file sample, ascending by size, one process per
run:

| files | source bytes | wall | exit |
| --- | --- | --- | --- |
| 1 | 372 | 63.1 s | returned `A` |
| 10 | 5,332 | 67.5 s | returned 6 A / 4 R |
| 40 | 388,527 | 187.3 s | **`EXIT=137` — OOM-killed after 34 files** |

Two numbers fall out, and they point at different walls:

- **Time is not the wall.** 63.1 s → 67.5 s for nine more files is ≈ **0.49 s marginal per file**
  against ≈ **62.6 s fixed overhead**. That fixed cost is itself the shape under discussion: it is
  paid to stand the whole corpus up before any question is asked.
- **Memory is the wall, and it is not about big files.** The kill came at file ~34 of 40 with only
  **94 KB of source consumed**, on files of ~7.8 KB each — the 212 KB outlier sorts last and was
  never reached. So ~94 KB of parsed source exhausted the runner's ≈1.6 GB budget. Whether that is
  parse-tree retention, interpreter heap growth, or both is **not** established by this probe and
  is not claimed here; what is established is that the process cannot hold the result of parsing
  34 small files, against a census subject of 3,733 files and 31.3 MiB.

The census fold accumulated only a short result string, so the retention is not the probe's
accumulator. A corpus-grain fold over this route does not fit in a process today.

### The verdict: outcome 3, and DESIGN has already rejected this shape once

Recorded here because it converts a performance observation into a repository ruling. DESIGN §6's
lens bullet records the deletion of the corpus-wide censuses in #8140 (2026-08-11) and states the
defect in general terms: *"the unit of computation was the world, the unit of fact was one
module's authorship, and the price was paid by every consumer that wanted a witness roster and
nothing else."* Its declared next-rung trigger is the seed-side shape: an authorship fact
*"belongs on the module's own declaration, checked at ingestion where the module is parsed
anyway — one module's facts from one module's source — rather than reconstructed corpus-wide by a
consumer that wanted something else."*

That is axis 1's remedy with a precedent and a cost argument already attached. A `ServiceItem`
accessor plus a non-lossy body projection pays per module at the point the module is *already*
being parsed; a corpus-wide parse fold re-parses the world on every run to answer a question about
shell routes — 62.6 s of fixed world-acquisition before the first question, and an OOM before the
34th file.

The measurement lands on outcome 3, so this is not a hypothetical: the corpus-parse route is the
exact cost-shape defect this repository deleted machinery over, and it additionally fails the
correctness axis today. **The recommended increment is therefore the ingestion-side projection,
not a census-side fold**, and 0A rebases onto it:

1. a `ServiceItem` accessor carrying operations, transports, argv and stdin channels (axis 1);
2. a non-lossy body projection — total over statements (axis 2), labelled argument edges (axis 3),
   arm and occurrence identity (axis 4), resolved callee identity with a node-kind discriminator
   (axis 5);
3. a corpus-grain denominator that is an enumerated file set rather than an import closure
   (axis 6).

`parse_grammar_choice_overlap_residue` is worth filing separately regardless of which route wins:
it is a single named grammar deficiency refusing a large fraction of authored source in the
compiler's own parser, currently invisible because the only code path that would surface it —
`canonical_dag_source_parse_print_law` — has no callers.
