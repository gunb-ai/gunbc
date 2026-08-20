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

## 2. Five axes, five remedies

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

## 5. A specification-without-execution specimen found on the way

The full-fidelity route does exist in `.dag`: `v2.compiler.source_authority`
`parse_dag_source_ast` takes a `Medium<String>` to `Outcome<DagSourceAst>` where
`ParseTree = Node`, with `normalize`/`resolve` above it giving `ResolvedTree = Node`; and
`Filesystem.List`/`Filesystem.Read` are callable from `.dag`
(`v2.workflow.floor_discovery_producer` already walks scan dirs with both).

But `parse_dag_source_ast` has **no consumer**. Its only two call sites are inside
`canonical_dag_source_parse_print_law`, and that function's name occurs exactly once in the
entire tree — its own definition. It has no callers at all. It is a parse/print law that nothing
executes: DESIGN §5's specification-without-execution class, sitting in the compiler's own source
authority. So "the route exists in `.dag`" is not evidence that it works on the real corpus, and
measuring it is the only thing that can establish it. That specimen is worth a row wherever the
enforcement-intent registry ends up, independent of this census.

The measured cost signal is adverse and is recorded here rather than left to be rediscovered:
`v2.compiler.ingested_fixture_arrows` drives the same tokenize→parse→normalize→resolve path on
`"module m\n\nfn add(x: Int, y: Int) -> Int { x + y }\n"`, and its witnesses are enrolled in the
**long** lane because that three-line fixture exceeds the 5s fast-lane eval budget
(`wave1_gate1_long_witness_note`). The census subject is 3,733 `.dag` files and 761,844 lines.
Whether the route can carry a whole-corpus census, or only a fixture-grain one, is a measurement
and not an inference — it is the open question this finding hands back.
