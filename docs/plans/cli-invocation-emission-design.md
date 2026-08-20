# CLI invocation emission — the tree is the authority, argv is emitted

**Status: DESIGN NOTE PLUS ITS FIRST CUT.** An earlier revision of this line read "No code lands
from this note yet" and kept reading that way while the branch carried a carrier, a seed
realization, witnesses and one migrated production site. That is the stale-citation class in the
lane's own authority: a status sentence nobody edits because nobody is touching the paragraph
around it. What has landed is stated at each wave below; what has not is stated as not. Design-note-first, per the
hollow-alias precedent. The first vertical is specified here; nothing is built until the
note is accepted.

**Lane scope, narrowed (raised by the `shell → dag` session, accepted):** this note governs
**CLI-backed host effects only** — effects whose realization is a process invoked with an argument
vector. It is *not* a universal effect model, and the distinction is load-bearing rather than
pedantic: a Redfish/REST path reaches a modeled request and must reach **no** `JqInvocation`, no
`CliSurface`, no `ProcessArgvExpansion` and no shell target at all. Treating every host effect as a
CLI invocation would re-import at the effect layer exactly the mistake this lane is removing at the
argv layer — one realization mistaken for the interface. A negative falsifier witness proving a
REST path reaches none of these carriers is **owed by this lane and not yet built.**

**Lane:** supersedes the Lane B destination of `gunbc.plans.transport_argv_anemia_dissolution`.
That plan's *diagnosis* stands — "the model is the request; the curl argv is one derived
realization of it" — and its Lane A (HTTP → `transport rest`) is substantially right. Its
Lane B destination is wrong: it terminates at typed `extdeps/tools/*` operations and
`ArgvCommand` builders, both of which still contain literal argv. The plan therefore stops
one layer short.

---

## 1. The defect

An invocation is authored today as a list of opaque strings:

```
argv: ["jq", "-er", "--argjson", "index", "{index}", ".data[0] | keys | .[$index]"]
```

Three independent facts are welded into that one list:

1. **what jq is** — a program operand, a raw-output mode, typed variable bindings, an exit policy;
2. **how we reached it** — locally, or over `sshpass`+`ssh` to a BMC;
3. **what we asked it** — an OpenBMC-specific projection over fan PID zones.

Because they are welded, every new question is a new hand-typed list, and each list
re-decides all three. The consequence is not stylistic. A flag nobody modeled is a flag
that is trivially droppable, which is how `--fail` went missing from eleven curl sites and
made a 401 read as success.

### The census (measured in-tree, not inherited from the plan)

| fact | count |
|---|---|
| `argv:` lines | 597 across 129 files |
| `transport shell` blocks | 250 |
| `transport rest` blocks | 95 |
| `operation` declarations | 322 |
| operations declaring `success: Bool from "exit_success"` | 173 |
| operations declaring `exit_code: Int from "exit_code"` | 69 |
| `shell_quote` call sites | 36 |
| argv lines with a literal program head | 282 |
| …of those, heads that are **wrappers**, not tools | 78 |
| `sh -c` / `bash -c` embedded-language leaves | 14 |

Two precision bounds: 315 of 597 argv lines have computed heads and are unclassifiable by
this method, so 78 is a **lower bound** on wrapper-hidden tools; the metacharacter count is
a text match, not a parse.

**The plan's own census is stale by ~2.3×** — it records 262 argv lines across 37 files, and
148/51 for the success/exit_code split. Both measured above.

### The census cannot currently be taken

Counting by argv head yields `git` 43, `sshpass` 40, `sh` 13, `sudo` 9, `env` 9. The middle
four are not tools; they are wrappers, and the real program sits at some positional offset
inside the list. All seventeen jq-over-ssh sites count as `sshpass`, not as jq.

This is itself the argument. Tool identity is a positional accident inside a flat string
array, so the corpus cannot answer "what do we invoke" without knowing that `argv[11]` is
the real program because `argv[0..10]` happened to be an ssh prefix.

---

## 2. Facts, separated by kind

### 2a. Substrate facts (read from source)

- `v2.std.compilers.target_model` `TargetModel` is target-agnostic in its bundle/lex/spellings
  shape, but **text-specialized** in three fields: `binding_spellings: Map<Symbol, String>`,
  `token_class_emit_transforms: Map<Symbol, fn(String) -> String>`, `authority_source_text: String`.
- The emit carrier is `v2.std.compilers.target_model` `TargetText { source: String }` — one
  String→atom introduction, concatenation as the only join, whole-value render as the only exit.
- `v2.extdeps.formats.sql_target` proves a **non-programming-language** target registers through
  `target_model_make_with_emit_transforms`. It does **not** prove carrier independence: its
  serialization node is still source-text shaped.
- The public emit seam commits to `Medium<String>`. `Medium<R>` itself is generic; the text lock
  is in `TargetModel`, `serialize_target` and `emit`, not in `Medium`.
- `dag/extdeps/languages/` holds 32 language authorities. **None is a CLI-invocation authority.**
- `extdeps.exec.command` renders argv as `join(map(argv, shell_quote), " ")`, and is anchored to
  the **SSH** manual while defining `curl`, `mkdir`, `tar`, `make`, `sed`, `cp` builders — plainly
  a cross-tool realization utility, not the authority for those tools' semantics.
- `v2.std.host_transport` `ProcessInvocation` / `InvocationArg` exist with 72 consumer references.
- `v2.extdeps.posix` `Command`, `PosixArgument`, `SpawnProcessRequest` exist with **zero consumers
  outside the declaring module**.
- `extdeps.posix.shell_command_language` exists as its own subject.
- `gunbc.command_runner` carries `command_runner_scaffold: Disposition = Scaffold`.

### 2b. Upstream authority facts (cited)

- **IEEE Std 1003.1-2024, Base Specifications Issue 8, §12** — 14 numbered Utility Syntax
  Guidelines; `--` ends options; options precede operands (G9); `-` denotes stdin (G13);
  option grouping behind one `-`; option-arguments as separate tokens.
- **POSIX `sed`** declares an explicit deviation: the relative order of `-e` and `-f` is
  significant. So POSIX is the *base* grammar, never the whole tool grammar.
- **RFC 4254 §6.5** — the SSH `exec` channel request carries a single `command` field of type
  **string**. No argv vector exists in the request, so argv structure cannot cross that boundary
  as structure. The protocol does *not* define that string as shell source; shell re-parse is an
  OpenSSH implementation fact, as `extdeps.ssh.session` already records.
- **IEEE Std 1003.1 Shell Command Language** — grounds the shell leg. Already cited by
  `extdeps.posix.shell_command_language`, whose own note reserves it for exactly this use:
  a POSIX-shell target "registers against THIS declaration and needs no new grounding."
- **jq manual** (`jqlang.org/manual/`) — `--raw-output`/`-r`, `--exit-status`/`-e`,
  `--argjson name JSON-text`, `--arg name value`, `--` terminator, file operands vs stdin.
- **RFC 7950 (YANG 1.1)** — abstract data model held separate from concrete encoding; one model
  renders as NETCONF XML or RESTCONF JSON. Cisco IOS XR ships "Model-Driven CLI" on that basis.
  Prior art: the networking industry ran this exact migration from CLI-scraping to modeled trees.
- **PowerShell 7.6** — parameters bind to typed .NET objects; parameter metadata (`Position`,
  `Required`, `Accepts multiple values`, `Accepts pipeline input`) is machine-readable. Evidence
  that a per-tool option table can be data rather than prose.

### 2c. Domain facts discovered during design (these constrain the model)

- `extdeps.bmc.openbmc_fan_control` declares `OpenBmcSensorValueAbsent` — a **third state**
  beside Refused and Observed. `openbmc_sensor_integer_projection_result` decodes empty stdout
  to it, and `dag/test/claim/bmc_typed_operations_witness_test.dag` witnesses it.
  **jq producing no output is a modeled value, not an error.** Any exit partition folding
  "no result" into failure breaks working, witnessed code.
- `gunbc.host_effect_realize` `bmcweb_token_extraction_verdict` refuses blank stdout on its own.
  So at that site `-e` was never the mechanism; it is transport *loudness*.
- Of the four jq operations in `openbmc_fan_control`, **all are live**:
  `ProjectFanConfig`, `ObjectMapperServiceCount`, `ObjectMapperServiceAt`, `SensorIntegerValue`.
  Only `extdeps.tools.jq` `jq.Json.ExtractRaw` is zero-consumer tree-wide.

---

## 3. The layering

A layer is only real if something varies at it. Checked against three CLI families:

| layer | owns | POSIX | PowerShell | Cisco |
|---|---|---|---|---|
| **intent** | the tree: subject, named params, operands | *identical across all three* |
| **vocabulary** | which params exist, types, cardinality, positional, exclusivity | tool's manual | cmdlet parameter metadata | YANG module |
| **surface grammar** | how a bound param renders | `-a`, `--long=v`, bundling, `--` | `-Name value` / `-Name:value` | keyword paths in a mode hierarchy |
| **value encoding** | how a typed value becomes a param value | bytes; nested languages embedded | .NET objects — no serialization | typed per YANG leaf → XML/JSON |
| **carrier** | what the emitted thing physically is | argv vector *or* shell text | in-process object pipeline | NETCONF RPC document |
| **transport** | where it runs | local exec, ssh, enable-mode session, REST |

The families agree completely at *intent* and disagree at *surface grammar*, *value encoding*
and *carrier*. That is what makes intent the authority and everything below a rendering.

### Two consequences

**Carrier ≠ surface grammar.** Emitting to shell text when the destination is `execve` invents
a quoting problem that does not exist. `shell_quote` at 36 sites is the current mitigation for
that conflation, not a requirement.

**Value encoding nests.** A jq program is itself a language embedded as a leaf in a POSIX
argument. So is `sh -c` — 14 sites. A nested-language leaf should be a typed sub-tree, not an
opaque String. Modeling jq's *expression* language is a separate vertical from modeling its
*invocation* structure, and is explicitly not a prerequisite here.

---

## 4. The model

```
JqInvocation                    semantic axes only; no flag spellings reachable
    |
    |  jq cited option rows  +  shared CLI grammar rows
    v
CliInvocationTree               options grouped with their values; operands; terminator
    |
    |  emission — roles erased here, correctly
    v
List<PosixArgument>             argument boundaries structural; no quoting
    |
    +-- local:  tool resolution -> extdeps.posix Command -> spawn
    |
    +-- ssh:    -> shell simple-command tree -> shell text -> RFC 4254 command string
```

### 4a. Semantic layer

```
type JqProgram { source: NonEmptyStr }        // opaque, upstream-grounded, for now

type JqBinding
  = JqJsonBinding { name: JqBindingName, value: Json }
  | JqTextBinding { name: JqBindingName, value: String }

// JqBindingName is jq-owned, not a bare NonEmptyStr. Duplicate-name policy is declared,
// not left to jq's last-wins behaviour: the admission fold refuses a repeated name.

type JqInput
  = JqInputFile  { path: FilePath }
  | JqInputStdin { content: String }

type JqOutputEncoding = JqJsonOutput | JqRawOutput   // -r writes STRING results raw;
                                                     // non-string results stay JSON-formatted

type JqExitPolicy = JqProgramExit | JqLastResultTruthiness

type JqInvocation {
  program: JqProgram
  bindings: List<JqBinding>
  input: JqInput
  output_encoding: JqOutputEncoding
  exit_policy: JqExitPolicy
}
```

`JqInput` is where stdin stops pretending to be an option: `JqInputFile` produces one file
operand, `JqInputStdin` produces **no operand** and sets process stdin. That split is load-bearing
here — the four local sites are stdin-fed, the seventeen remote ones are file-operand.

### 4b. Observation: two orthogonal axes, sealed

Two earlier revisions of this section were wrong, in the same direction each time — putting on
one axis a fact that lives on two.

**Termination and output presence are orthogonal.** The live `SensorIntegerValue` specimen proves
both can hold at once: exit status 0 **and** zero results produced. It runs `-r` without `-e`, its
filter takes the `empty` branch for a non-number, and that run exits zero with empty stdout. So
`JqExitZero | JqProducedNoResult` is not a coproduct — it is a product, and writing it as a sum
makes the live case unrepresentable.

Verified against `jqlang.org/manual/`:

```
default             exit 0 when the program runs successfully, REGARDLESS of output
--exit-status / -e  exit 0  last output neither false nor null
                    exit 1  last output false or null
                    exit 4  no valid result was ever produced
halt_error          program-chosen, default 5      usage 2      compile error 3
```

```
type JqExitDisposition
  = JqExitZero
  | JqExitFalseOrNullUnderExitStatus      // JqLastResultExit only
  | JqExitNoResultUnderExitStatus         // JqLastResultExit only
  | JqExitRefused { code: Int }

type JqOutputPresence
  = JqOutputAbsent
  | JqOutputPresent { stdout: String }

type JqObservation sole_constructor {
  process: ProcessObservation      // raw evidence, retained
  exit: JqExitDisposition          // decoded projection, bound to it
  output: JqOutputPresence
  stderr: String
}
```

| case | exit disposition | output presence |
|---|---|---|
| `SensorIntegerValue`, numeric | `JqExitZero` | `JqOutputPresent` |
| `SensorIntegerValue`, non-numeric | `JqExitZero` | **`JqOutputAbsent`** |
| `ObjectMapperServiceAt`, found | `JqExitZero` | `JqOutputPresent` |
| truthiness policy, no result | `JqExitNoResultUnderExitStatus` | `JqOutputAbsent` |
| jq execution error | `JqExitRefused` | independently observed |

**The decoder is policy-indexed and is the only mint.**
`decode_jq_observation(invocation, process) -> JqObservation` — not a global `exit_code -> JqExit`
lookup, because codes 1 and 4 mean nothing under `JqProgramExit`, and zero results under
`JqProgramExit` still produce code zero.

**Sealing is required, not decorative.** Rejecting `success: Bool` beside `exit_code` removes one
writable contradiction; leaving `JqObservation` freely constructible recreates it under richer
names — `{ exit: JqExitZero, exit_code: 4 }` would be authorable. `sole_constructor`, minted only
by the policy-aware decoder, closes it. The raw process observation is retained as evidence and
the decoded facts are a projection **bound to it**, never two caller-authored fields side by side.
This is the first concrete place where the answer to the rung question is *yes, a construction
wall is available today*.

**A stdout `String` cannot generically prove "exactly one nonempty raw string".** jq emits a
*stream*; under `--raw-output` a raw string may contain newlines, which is why `--raw-output0`
exists ("Like `-r` but jq will print NUL instead of newline after each output"). So `foo\nbar\n`
is ambiguous between one string containing a newline and two results, and a collapsed stdout
`String` cannot separate them. This does not block the first vertical — an integer has an
unambiguous textual grammar — but **this note promises no generic
`decode_exactly_one_nonempty_string`.** Lifting that requires one of: a single JSON envelope
parsed as JSON; `--raw-output0` once jq version/capability is modeled; or a program collecting its
result into a one-result container the domain decoder checks.

**No `success: Bool`.** The seed derives it as exactly `exit_code == 0`, so the pair carries one
fact twice. `extdeps.git.git` `ls_remote_reports_termination_not_success_note` establishes this;
173 operations still carry the Bool.

### 4c. Two-level row derivation — the keystone

Both the **selection** of an option and its **spelling** must be row-derived:

```
JqLastResultTruthiness  -> JqCliOptionExitStatus                 (prevents omission)
JqCliOptionExitStatus   -> canonical "--exit-status", alias "-e" (prevents bespoke spelling)

JqJsonBinding { name, value }
    -> [ argument "--argjson", argument emit(name), argument emit(value, Json) ]
```

If only the second level is data-driven while a function still says "when policy is X, append
option Y," the droppable-option class survives one layer down. That is the difference between
this design and centralizing 597 literals into N helper functions.

So the precise statement replacing "`-e` is part of the contract, not a flag" is:

> `JqLastResultTruthiness` is part of the contract. `-e` and `--exit-status` are concrete
> spellings of that property in the jq CLI realization.

### 4d. Grammar decomposition

```
shared CLI grammar          option, option-argument, operand, repeated group,
                            terminator, short-option cluster, long spelling,
                            joined vs separate value

+ per-tool cited rows       jq options and operand order
                            sed's -e/-f ordering exception
                            git subcommand grammar
                            tar mode-word grammar
                            ssh -o forms
```

Named `CliSyntax`, not `PosixCli`: real tools deviate, POSIX `sed` provably so, and the name
should not assert conformance the corpus cannot claim.

### 4e. The structured CLI tree — why roles live here and not in the arg vector

```
type CliElement
  = CliOption  { option: CliOptionRef, values: List<CliValue> }
  | CliOperand { value: CliValue }
  | CliOptionTerminator
```

An option's values cannot detach from their option: `--argjson`, its binding name and its JSON
value stay one subtree until the grammar lowers them.

**Role and provenance are orthogonal axes and must not share a coproduct.** A workspace path may
be an option value *or* an operand; a produced artifact may be either; a literal may be an option,
subcommand, option value, mode word or operand. Fusing them makes `OptionValue ∧ WorkspacePath`
unrepresentable, and expanding to `WorkspacePathOptionValue`, `WorkspacePathOperand`, … is exactly
the combinatorial growth the grammar exists to remove.

**`CliOption { values: List<CliValue> }` is too permissive as a *public* carrier.** Freely
constructible, it admits `--argjson` with zero, one or three values; a flag receiving a value; a
non-repeatable option used twice; options in an illegal order; an option belonging to a different
tool; operands before options where the tool forbids it. The cited row must therefore carry
option identity, canonical spelling, aliases, argument arity, repeatability, placement discipline
and joining discipline — and the emitter must accept only a **normalized, admitted** tree:

```
type CliOptionSchema {
  option: CliOptionRef
  arguments: CliArgumentSchema
  repeatability: CliOptionRepeatability
  spelling: CliCanonicalSpelling
  placement: CliOptionPlacement
}

type AdmittedCliInvocation { tool: CliToolRef, options: List<AdmittedCliOptionUse>, operands: List<CliOperand> }
```

Domain callers receive **no constructor** for `AdmittedCliOptionUse`. The jq semantic-to-syntax
projection selects the row and supplies values; the admission fold checks the schema. This is the
`sole_constructor` + `admit_callers` shape that already sealed `TransportScript` in this corpus,
and it is what raises the rung for the migrated path — see §9.

Roles are then **deliberately erased** at emission, because the operating system receives an
ordered vector of strings. That erasure is emission, not anemia. The present defect is only that
no preceding tree exists — authors write the vector directly.

### 4e-bis. The process plan is not the CLI grammar

`CliSyntax` owns how options and operands become argument boundaries. It owns **nothing else**:
standard input, environment, working directory and descriptor routing belong to the *process
plan*.

```
type JqProcessPlan { command: CliCommandSurface, stdin: ProcessInput }

JqInputFile(path)     -> one CLI operand,  no jq-data stdin
JqInputStdin(content) -> no file operand,  stdin = content
```

This is why `SensorIntegerValue` is a good first proof: the input content must stay **outside
argv** while the jq program must stay **one argv argument**. Two properties an argv-only model
cannot even state — and precisely the gap `extdeps.llm.cursor_cli` hit when an argv-shaped
carrier could not hold an environment-passed credential.

### 4f. Carrier: argv is native, text is the derived special case

POSIX utility syntax operates over *arguments*. Shell syntax is the preceding language that turns
words and expansions into those arguments; quote removal happens before the utility sees them.
Therefore the argv vector is the higher-structure carrier and shell text is a serialization of it
through an additional language.

Making text canonical and re-deriving argv by parsing would be backwards: it forces the strongest
local transport through an unnecessary shell grammar, quoting discipline and injection surface,
because one weaker remote transport loses structure.

**Local and SSH are not two jq targets.** They are one utility target, two realization carriers,
and one *additional nested target* on the SSH path — the POSIX shell grammar, whose quoting and
parsing rules are modeled independently and are already grounded by
`extdeps.posix.shell_command_language`.

The counterfactual that proves command text is not intrinsic: a remote receiver accepting a
structured request and calling `execve` itself would eliminate the shell leg while keeping SSH
as transport.

**Existing precedent for the composition shape:** `effect_plan_bash_materialize` already obtains
an argv vector, converts it to Bash command nodes, then serializes the Bash grammar. Its argv
source is still the old declaration-owned materializer, but the shape is right.

### 4g. Physical layer

Lower into `v2.extdeps.posix` `Command` / `PosixArgument`, **not** `v2.std.host_transport`
`ProcessInvocation`. The latter is an emit-harness carrier whose `InvocationArg` variants
(`LiteralArg | WorkspacePath | ProducedArtifact`) classify build-workspace provenance, not CLI
roles; `build_transport_admission` only ever asks whether a step reads a declared workspace
resource, and `emit_host` flattens all three variants back to text.

Evidence that its provenance vocabulary is hand-applied rather than derived: `python.dag` models
its script as `WorkspacePath`, `go.dag` models `main.go` as `WorkspacePath`, and `c.dag` models
`fixture.c`, `-o` and `fixture` as three undifferentiated `LiteralArg`s — so C's compiler read of
`fixture.c` is invisible to the admission fold. That is a real existing harness defect, and an
independent reason not to promote this carrier to the universal process boundary.

`ProcessInvocation` is also missing process-wide axes already needed elsewhere: **no environment,
no stdin, no working directory, no file-descriptor mapping.** That omission has already been
discovered independently and forked around: `extdeps.llm.cursor_cli` minted its own
`CursorProcessInvocation { program, argv, environment }` rather than reuse it, and its note states
the general principle with a security consequence attached —

> an argv is not a process. A process is a program, an argv AND an environment, and a credential
> passed by environment is invisible in a model whose output stops at the argv — there was no
> field for it to appear in.

A fork in `dag/extdeps/llm` against a carrier in `src/v2/std` would not have been necessary if
`host_transport` `ProcessInvocation` were the general authority its name suggests. That is
independent corroboration of the name collision, arrived at by a different lane for a different
reason.

**What `ProcessInvocation` remains good for.** It is not deleted by this lane. It stays as a
downstream adapter for the emit-host harness — fixed option spelling to `LiteralArg`, workspace
file operand to `WorkspacePath`, generated binary operand to `ProducedArtifact` — so the harness
keeps its effect provenance *after* CLI emission. Two constraints: no jq or domain caller
constructs those variants directly, and jq does not route through the emit-host harness merely
because that type already exists. The honest longer-term cleanup is renaming it and
`InvocationArg` to name their real scope (`EmitHostProcessInvocation`,
`EmitHostArgumentMaterialization`), which is a separate replacement migration and explicitly not
bundled here.

**Generalizing that subsystem is NOT a prerequisite for the first vertical.** Making stdin, cwd
and fd mapping land on `Command`/`SpawnProcessRequest` before any jq migration turns one proof
vertical into a process-runtime project. The stable boundary is `CliCommandSurface + ProcessInput`;
it binds first through one narrow local jq realization whose entire content is the jq executable
identity, a splice of already-emitted arguments, and the process channel wiring — **no option
spelling and no domain filter**. That is an honest realization boundary, not another argv
authority. Its dissolution trigger is explicit: it dissolves when the same emitted surface binds
to `extdeps.posix` `Command`. Emit a `CliToolRef` and let each realization resolve `"jq"` or an
absolute path.

`v2.extdeps.posix` `Command` is the right conceptual layer, with four bounded deficiencies:

1. `program: AbsolutePath` — a semantic invocation names a *tool*; resolution must produce the
   path at realization time.
2. no stdin or file-descriptor actions on `Command`/`SpawnProcessRequest`.
3. `PosixByteString = FreeMonoid<Byte>` does not visibly exclude embedded NUL, which POSIX
   process arguments require.
4. no executed proof that the active `dag/` lane can consume this `src/v2` carrier.

---

## 5. Ownership

| owner | owns |
|---|---|
| `extdeps.tools.jq` | what raw output means, what exit-status mode means, that `--argjson` takes a name and a JSON value, which options repeat — and **which spellings exist and which is canonical** (`long = exit-status`, `short alias = e`, canonical preference, value cardinality) |
| shared `CliSyntax` | **how a chosen form is constructed** — a long name renders `--` + name, a short renders `-` + character, clustering where the tool row permits, joined vs separate values, option/operand/terminator order |
| `gunbc.bmc_fan_projection` | the `TEMP_SOC` policy, the fan curve, controller cardinality, the desired topology |
| `extdeps.bmc.*` | programs projecting a documented OpenBMC response or config field |
| realization | local vs SSH vs another handler |
| **no caller** | `-e`, `-r`, `--argjson`, `--`, their positions, or the choice between `-e` and `--exit-status` |

The tool row owns *which forms exist and which is canonical*; the grammar owns *how a chosen form
is constructed*. An earlier revision gave both authorities "canonical spelling" — two authorities
for one decision, the §3 violation this note is about, committed inside it.

The split is by **what fact the program encodes**, not by the fact that jq executes it.
`ProjectFanConfig` embeds `TEMP_SOC`, the aggregate fan PID entry, desired inputs, minimum and
failsafe duties, hysteresis and exact-one-controller policy — gunbc's chosen topology, so it moves.
`SensorIntegerValue`, `ObjectMapperServiceCount` and `ObjectMapperServiceAt` interpret **upstream
OpenBMC response shapes**; they stay under `extdeps.bmc` even though jq currently realizes them.
The several *remote* fan-query programs carrying a literal `TEMP_SOC` are the ambiguous middle:
they belong beside the projection authority, or should derive that value from it, rather than
keeping the literal in `extdeps`.

Filters leave both `extdeps.tools.jq` (business policy in extdeps is a layer inversion) and
`openbmc.PasswordSshTransport` (SSH does not own what the remote program means). They do not all
land in one generic "workflow layer": policy-bearing ones go to `gunbc.bmc_fan_projection`, which
already owns the board, firmware, curve, config path, controller entry and tach inputs.

---

## 6. Rules

1. `-e`, `-r`, `--argjson`, `--` and their aliases may appear **only** in cited concrete-syntax rows.
2. Semantic modules construct no option node and no argv string.
3. Per-tool lowering contains no bespoke option-selection fold; semantic variants select grammar
   rows exhaustively.
4. Transport consumes an inner invocation. SSH may **not** be modeled as `append(ssh_prefix, inner.argv)`.
5. The CLI inverse consumes `List<CliArgument>`, not shell text — and it is **partial and
   ambiguity-aware**, not bijective: clustered short options, optional option-arguments, operands
   beginning with `-` and tool deviations admit multiple parses, so it answers
   `CliParseUnique | CliParseAmbiguous | CliParseRefused`. It is **not** a Wave 1 acceptance
   criterion; current argv remains an offline behavioral oracle. Shell text is parsed by the shell
   grammar first; only a static literal simple command projects back to a CLI surface.
6. Every vertical deletes its prior argv authoring site **in the same cut**. "New tree plus old
   helper for compatibility" creates two authorities.
7. No new production function returns `List<String>` or `ArgvCommand` containing tool flag literals.
8. `ArgvCommand` builders and `transport shell { argv: [...] }` are migration substrates, not the
   final authority.

---

## 7. The cut

Per the §3 replacement-migration rule, stated at exact identity grain.

**Authority being replaced:** argv-as-authored-authority for jq invocations.

**Root consumer × subject population:** the four live `openbmc.JsonProjection` operations
(`ProjectFanConfig`, `ObjectMapperServiceCount`, `ObjectMapperServiceAt`, `SensorIntegerValue`)
plus the seventeen `openbmc_password_ssh_transport` remote jq operations.

**Recut: the first vertical is LOCAL only.** An earlier revision put local execution and SSH in
one "smallest complete" vertical. That is not smallest — SSH adds a second language target, RFC
4254 string loss, portable-word refusal, quoting, and the latent `sshpass -d 0` collision. The
unit is **one local jq vertical, demonstrated by one feature-rich migration and one discriminating
counterexample.**

**Wave 0 — carrier probe.** Stacked with its first production consumer, never mergeable alone.
A probe-only non-text serializer beside the text one; sealed `CliSurface`; existing text emission
byte-identical; argument boundaries observable *by execution*; direct/cast/unadmitted-mint REDs;
explicit refusal when no CLI serializer is wired. See §10 — this probe must execute, because it
will not refuse at compile time.

**WAVE ORDER CORRECTED BY EXECUTION: 1B CUT FIRST, and the note was wrong, not the branch.**
Wave 1A was named the strongest first exemplar because it exercises the most axes at once —
including a typed JSON binding through `--argjson`. That is exactly why it cannot be first: the
binding lowering is **not built**, and `jq_invocation_lower` refuses it as
`JqBindingLoweringUnwired`. Choosing the richest site as the first cut selected the one site whose
lowering is unimplemented. `SensorIntegerValue` (Wave 1B) migrated first because its every axis is
wired: program operand, stdin input, raw output, and `JqProgramExit` selecting no exit-status
option. Its falsifier role is undiminished by going first — a counterexample does not require the
thing it falsifies to exist yet. 1A follows once `--argjson` lowers.

**Wave 1A — `ObjectMapperServiceAt`.** The strongest first exemplar, because its existing argv
exercises nearly every load-bearing axis at once: `-e` truthiness exit, `-r` raw output, a typed
JSON binding through `--argjson` (an option with **two** values), stdin input, a jq program
operand, exact nonempty-string domain decoding, and a live recursive consumer. After migration the
OpenBMC caller stops reading `success`/`stdout`/`stderr` itself and matches
`OpenBmcServiceObserved | OpenBmcServiceProjectionRefused`. No layer above jq's cited rows can
name `-e`, `-r`, `-er` or `--argjson`.

**Wave 1B — `SensorIntegerValue`, the required falsifier.** Immediately stacked, *before* the
abstraction is advertised as ready for repetition. It is the counterexample to a design overfit to
`-e`, and it proves four things Wave 1A cannot: that `JqProgramExit` selects **no**
`JqCliOptionExitStatus`; that `JqOutputAbsent` is not automatically a refusal; that termination
and output presence really are orthogonal in the observation model; and that one lowering supports
two opposite domain policies unchanged. Its acceptance population:

```
{"data": 41.6}      -> Observed 42        {"data": "41"}     -> Absent
{"data": 41}        -> Observed 41        {"data": null}     -> Absent
{}                  -> Absent             invalid JSON       -> Refused
two numeric inputs  -> Refused, never first-pick              jq nonzero -> Refused
```

**Wave 2 — first SSH migration.** Only after the local vertical is green: `CliSurface` → shell
simple-command tree → shell grammar emission → RFC 4254 command string, reusing the grammar-owned
quoting the effect-plan Bash path already demonstrates rather than retaining `shell_quote`. Owns
the portable-word limitation, file-input-only admission, and a typed refusal for stdin-fed remote
jq while `sshpass -d 0` holds fd 0. Then the remaining remote population is mechanical.

Emission-side witnesses, proven separately from semantics at every wave:

```
the semantic property selects the option identity     (row level 1 perturbation)
that identity selects its canonical spelling          (row level 2 perturbation)
JqProgramExit emits NO exit-status option
the jq program is exactly one argument
stdin content appears NOWHERE in argv
argument order is stable
direct / cast / unadmitted-mint construction refuses
the old transport argv site is absent in the same cut
no success: Bool survives on the new path
```

Both row levels need independent perturbation controls: a single spelling perturbation proves
concrete spelling is authoritative but says nothing about whether a required semantic property can
be omitted.

**Do not make the first vertical solve SSH.** Local exemplar and transport composition are
separate proofs.

**First vertical (smallest complete):**

1. `JqInvocation` and its semantic axes.
2. `CliInvocationTree`.
3. Shared CLI grammar rows + jq's cited option rows.
4. Emission to `List<PosixArgument>`.
5. Local resolution into `extdeps.posix` `Command`.
6. A process-input realization for jq stdin, separate from argv.
7. One live jq population cut over, with its prior argv authoring site deleted.

The SSH handler is **Wave 2 and is deliberately absent from this list.** An earlier revision
carried it here as item 7, directly contradicting "Do not make the first vertical solve SSH" two
paragraphs above and the recut in §7 — the same list could be cited to prove either scope. SSH adds
a second language target, RFC 4254 string loss and the `sshpass -d 0` collision; none of that is
needed to prove a local vertical.

The carrier lands **with** its first live consumer, or in an immediately preceding stacked PR that
already contains an executing consumer. A standalone carrier PR with no consumer would reproduce
the current unconsumed `v2.extdeps.posix` situation exactly.

**Deleted at cutover:** `extdeps.tools.jq` `jq.Json.ExtractRaw` (zero consumers — deleted first,
not kept beside the new authority); the migrated sites' `transport shell` argv literals; the
seventeen re-inlined `sshpass`/`ssh` prefixes.

**Explicitly NOT in this cut:** adding variants to `InvocationArg`; updating its 72-reference
population; renaming the harness carrier to `EmitHostProcessInvocation`; repairing the C/Go/Python
descriptor inconsistency; making `HostTransportDescriptor` a general process authority; modeling
jq's expression language; the other 575 argv lines. Those are independent lanes.

**Deliberately not built:** a coverage lens over argv arrays. Under the replacement-migration
rule that is a census over a structure being deleted — the deletion is the census, since every
real dependent refuses loudly. The plan proposes one; this note declines it.

---

## 8. Witnesses

Semantics and serialization are witnessed separately:

- **semantic** — the caller requested the correct jq behavior.
- **emission** — that semantic tree lowers to the expected argument vector.
- **perturbation** — changing the jq option row changes or refuses the emitted vector. This is
  the control that proves the rows are load-bearing rather than decorative.
- **domain** — absent, null, false, blank, wrong-type and multiple outputs cannot reach the write.
- **positive control** — one valid token still does.
- **absence control** — a non-numeric sensor reading still decodes to `OpenBmcSensorValueAbsent`
  and does not become a refusal.

---

## 9. Rung, by exact subject

An earlier revision said "mechanically preventable, because a caller can still construct a raw
`List<String>` and nothing yet refuses it." That sentence is internally inconsistent: **if nothing
refuses it, it is not mechanically prevented.** It is measured or mitigated, not prevented. It
also *understated* what the migrated path can reach. One number for several populations was the
error.

| exact subject | honest rung after the first vertical | mechanism |
|---|---|---|
| forging a `CliSurface` **by record literal or cast** | structurally impossible | `sole_constructor`, whose cross-module refusal is witnessed by the corpus `sole-constructor-cross-module` probes — not re-proved here |
| minting a `CliSurface` **whose content did not come from rows** | **mitigatable only** | *no wall.* `serialize_cli_arguments` is public and accepts caller-supplied `lex` and fragments, so any module can obtain a well-typed `CliSurface` carrying arbitrary text |
| omitting `--exit-status` from an accepted truthiness invocation | **structurally impossible within the new path** | exhaustive semantic→option selection; zero-or-many rows refuse; no mint on miss |
| hand-authoring flags inside a migrated route | **structurally impossible after cutover** | handler takes a semantic invocation, emits internally, old argv site deleted |
| reintroducing the exact old operation transport | mechanically preventable | structural deletion witness |
| a computed argument vector losing its argument boundaries at the host edge | **mitigatable only** | the `ProcessArgvExpansion` arm makes boundaries survive *on the carrier path* and refuses every malformed shape it is handed, but the guessing path it sits in front of is untouched for every value that is not this carrier — 21 operations still splice a declared `List` param. This is a repair at ONE seam, not a wall over the class |
| a domain module authoring jq program source | **mitigatable only** | §12 — `JqProgram` is public; nothing prevents it |
| decoding a jq exit code against the wrong contract | **structurally guaranteed on the new path** | §13 — the policy travels on the plan and `jq_classify_observation` is the only reader; a domain module has no exit code to misread |
| writing another raw jq argv elsewhere | **still writable** | unrelated raw shell/process routes remain |
| arbitrary raw argv corpus-wide | outside this vertical | later process/transport confinement |

So: **the first migrated jq path can be structural now; the repository-wide raw-jq class stays
open; the repository-wide raw-argv class is a later migration.**

**The seal is on the shape, not on the provenance — corrected 2026-08-19.** The row above
previously read *structurally impossible now — only the row emitter, serializer and decoder may
mint*, which is false and was the §4b inflation this document warns about, asserted about this
document's own subject. `sole_constructor` closes the record literal and the cast; it says nothing
about a **public fold that takes caller-supplied inputs**, and `serialize_cli_arguments` is
exactly that. An earlier claim in this lane that `CliSurface` was therefore *stronger* than
`TransportScript` had the comparison backwards: `TransportScript`'s mint is `admit_callers`-sealed
to two named production declarations, and `CliSurface`'s is not sealed at all.

**Ceiling and trigger, stated rather than left implied.** The obvious repair — `admit_callers` on
the serializer — is refused on §3 grounds: it would make `v2.std.compilers.cli_surface` name every
tool module that may emit a CLI, which is dispatch fused into the interface, the thing this lane
exists to keep peripheral. So the wall belongs on the **input**, not on the serializer: a sealed
`AdmittedCliInvocation` that only a tool's own row-derived lowering can produce, with the
serializer total over it. That is **not built**, and the design of its mint is genuinely open —
whatever seals it faces the same recursion one level up. Until it lands, provenance sits at
*mitigatable*, and the practical containment is that the only public jq entry point is
`jq_invocation_process_plan`, which is a convention, not a wall.

**Application-argument typechecking is not the terminal trigger.** The `04_infer` gap is real —
`explicit_return_conformance_note` records that conformance is judged only by the
ground-kernel-scalar and ground-element-collection discipline — but it is neither *necessary* for
sealing this path nor *sufficient* to eliminate alternate raw routes. It matters when a sealed
carrier crosses an open function boundary and correctness rests on ordinary argument conformance,
and the first vertical avoids that dependency: no arbitrary caller receives a public
`execute(surface: CliSurface)` to smuggle into. The domain caller passes a `JqInvocation`; the
handler emits and consumes the sealed surface internally.

The real terminal trigger for the broad jq class is a **dominator condition**:

> every production path capable of executing jq is dominated by the semantic jq handler, and raw
> process/shell routes cannot name jq except through an explicitly retained escape population.

General argument typechecking strengthens the boundary; it does not prove that property.

## 10. Open questions

1. **Carrier parameterization — and the probe will NOT refuse.** The exact current answer is not
   "unknown diagnostic." It is: **there is no carrier-mismatch refusal on this path, so a
   compile-only probe false-greens.** `emit`, `serialize_target` and
   `serialize_source_for_emitted` are concretely fixed to `Medium<String>`, and underneath them
   `target_serialize_source_from_model_bounded` returns `TargetText` rendered through
   `target_source_medium`. Passing a `cli_target` selects no other carrier — it still takes the
   text serializer. And `v1.04_infer` `explicit_return_conformance_note` records that declared
   return conformance is judged only by the ground-kernel-scalar and ground-element-collection
   discipline, so a structured mismatch such as `Outcome<Medium<String>>` vs
   `Outcome<Medium<CliSurface>>` sits outside the judged population and yields no diagnostic.
   **Do not treat a green compile as a positive result.**

   The smallest discriminating probe is therefore a **sibling concrete seam**, executed rather
   than compiled — `emit_cli_probe` binding `translate`'s target tree to a non-text serializer —
   asking one question: can `translate`'s output be consumed by a non-text serializer while the
   existing text serializer stays byte-identical? It does not ask the current `emit` to produce
   something its signature cannot express, and it does **not** start by genericizing `TargetModel`.

   Its controls: existing `emit` returns the exact prior bytes; the probe returns a sealed
   `CliSurface` whose argument boundaries are inspectable **by execution**; swapping the CLI
   serializer for the text one must *fail the test*, not compile-green past it; direct, cast and
   unadmitted-mint construction of `CliSurface` all refuse; and a missing serializer raises a new
   typed `TargetSurfaceSerializerUnwired` rather than silently falling back to source text.

   Only after that receipt does the seam choice get made. Prior: `TargetSurfaceSerializer<R>` is
   the smaller terminal seam, with the existing `emit` kept as a compatibility wrapper selecting
   the text serializer — parameterizing every `TargetModel` consumer before a non-text target has
   proven it necessary is the larger and less reversible move.

2. **`sshpass -d 0` and stdin.** The password occupies fd 0. A remote stdin-fed jq would collide.
   Latent, not live — all seventeen remote sites use file operands. Must be a typed refusal, not
   an accidental limitation. Options: remote file-input only; move password delivery to another
   descriptor; or a framed remote receiver.
3. **Portable-word allowlist.** `gunbc.typed_argv_exec` bounds what may cross SSH. jq programs
   contain spaces, pipes, brackets, parentheses and quotes — outside that alphabet. The shell leg
   must own canonical shell-word serialization with injection controls, or refuse.
4. Which of the 315 computed-head argv lines hide further wrappers.
5. **The remote jq rows do not preserve the jq program as one argument. Source-proven.**
   Traced end to end, with the local half verified in this tree:

   - `v1_interpreter.rs` `push_shell_argv_tokens` pushes each evaluated `String` **verbatim** — no
     splitting, no quoting — and both exec branches call
     `Command::new(&argv[0]).args(&argv[1..])`. So despite its name, `transport shell` at this
     boundary is **direct process execution**: there is no local shell, no `shell_quote`, no
     command-string renderer.
   - `sshpass` parses its own options, copies the remaining argument pointers, and `execvp`s them
     unchanged.
   - OpenSSH's client consumes the `--` after the destination as its **own option terminator**
     (not a remote quoting construct) and builds the remote command by appending each remaining
     argv member separated by one literal space, with no escaping.
   - OpenBMC's default SSH server is **Dropbear**, not OpenSSH — its `run_shell_command`
     invokes the user's login shell with `-c`.

   So `["jq", "-er", "[.zones[].pids[] | select(...)] | length", path]` arrives remotely as
   `jq -er [.zones[].pids[] | select(...)] | length /path`, and the remote shell reads the
   unquoted `|` as pipeline operators. The argument boundary is gone. The law the code needs is
   `shell_parse(join(map(shell_quote, argv), " ")) == argv`; what it currently assumes is
   `shell_parse(join(argv, " ")) == argv`, which holds only inside a portable-word alphabet these
   filters are outside — the same alphabet `gunbc.typed_argv_exec` exists to bound. Dynamic values
   such as `config_path` are unquoted too; the current path happens to be shell-safe, but
   `NonEmptyStr` does not establish that.

   **Split the judgments rather than calling the whole thing broken:**

   ```
   RemoteJqArgumentBoundaryPreservation   Violates — source-proven
   DeployedSshClientIsStandardOpenSsh     Unverified
   DeployedBmcServerUsesDropbearShellExec StronglySupported, not observed on this firmware
   AffectedOperationReachability          Unverified
   ObservedProductionManifestation        Unverified
   ```

   The saving conditions are enumerable and none is visible in the repository: a custom binary
   named `ssh` that shell-quotes (most plausible, since `sshpass` resolves through `PATH`); filters
   already carrying canonical outer quoting; a forced-command or structured receiver decoding an
   encoded argv; or a row whose program happens to be entirely portable words.

   **The executed control** uses the exact current prefix and direct argv execution — not a local
   shell — with `jq -n`, so it needs no stdin or file and cannot mutate BMC state:

   ```
   portable positive   ["jq","-n","-r","7"]                 -> exit 0, stdout "7"
   current form        ["jq","-n","-r","[1,2] | length"]     -> intended "2"
   serialized form     ["jq","-n","-r","'[1,2] | length'"]   -> expected "2"
   ```

   Discriminating result: portable and serialized controls succeed while the current form does not
   produce the same exit/stdout observation. Capture local client version, remote SSH banner and
   remote login shell. **If the current form unexpectedly yields `2`, that is positive evidence for
   an unmodeled saving layer**, and the next probe captures the actual command string seen
   remotely.

6. The SSH slice needs a discriminating program containing spaces, a pipe, quotes, brackets and a
   value containing a single quote, and must reuse the grammar-owned quoting the effect-plan Bash
   path already demonstrates rather than retaining `shell_quote`.
7. Reverse ingestion — reading argv back through the same rows — is the terminal second reading
   and is **not a prerequisite for the first emitter cut**.
8. **Measured substrate observation — `Optional<T>` at a primitive `T` in record-field
   construction position.** `stdin: Optional<String>` with an inline `Present { value: … }`
   refuses with `expected 'Coproduct(Optional)', got 'Primitive(String)'`, and the refusal is
   independent of the payload expression: a pattern-bound variable, a renamed binding, a
   shorthand pattern, and a bare `"x"` literal all reproduce it, while `stdin: Absent` resolves.
   Inline `Present` into an explicitly-declared `Optional<T>` field is ordinary elsewhere in the
   tree (`gunbc.witness_row_cost` `basis`, `gunbc.declared_import_closure_binding`
   `binding_source`, `v2.workflow.effect_plan_bash_materialize` `operation`), but every such `T`
   observed is a record or coproduct — so the discriminator is the **primitive type argument**,
   not the field form and not the explicit-vs-sugar spelling. This lane uses the corpus's own
   `String?` sugar, which is the existing modeled form for an optional field (625 sites) rather
   than a workaround around an unexplained error. **The observation is recorded, not diagnosed:**
   whether the sugar and the explicit generic genuinely differ at a primitive argument, or the
   refusal has another cause this bisect did not separate, is unestablished — and a §5 line-stop
   is owed on it before anything else in this lane relies on the distinction.

9. **A live Class B specimen, in this lane's own new code (found by review, 2026-08-19).**
   `extdeps.tools.jq` `jq_option_canonical_spelling` calls `cli_long_option_spelling` while the
   module's import list named only `CliArgumentSyntax`, `CliSurface` and `serialize_cli_arguments`.
   The call resolved anyway — and every Wave 1A assertion passed — because `cli_surface` was
   already in the assembled closure via those three imports. That is exactly the accidental
   coverage DESIGN's import-strip thread records as **blocking all further `dag/**` import
   stripping**: the binding came from pool membership, not from the bare-reference closure. Note
   which control does *not* catch this — the bystander probe above refuses a *qualified* import of
   a name a module does not export, whereas this is an *unimported bare reference* into a module
   that is present for other reasons. Different mechanism, and only the second is silent. Fixed by
   naming the symbol in the import list. **The general defect is untouched and not this lane's:**
   nothing refuses the next such reference, and a green witness is not evidence that a module's
   imports are complete.

9-ter. **BLOCKER, PROVEN BY EXECUTION: the argv transport silently destroys argument boundaries
   for computed lists (2026-08-19).** A `List<String>` built by folding is CONCATENATED into a
   single argv word; the identical list written as a literal is SPLICED. Same declared type, same
   elements, same handler, two different processes.

   **Receipt, through the production handler `jq.Process.RunWithStdin`:**

   ```
   arguments = ["--raw-output", "."]        literal   -> exit 0   (jq ran; two argv words)
   arguments = fold_list(... same two ...)  folded    -> exit 2
       jq: Unknown option --raw-output.
   ```

   And from the first wet run of the real vertical, three lowered arguments arriving as one:

   ```
   $ jq --raw-output--exit-status.missing   (exit=2)
   ```

   **Mechanism** — `v1_interpreter.rs` `push_shell_argv_tokens`. `Value::List` splices each item.
   `Value::Variant` tries `value_as_host_string` FIRST, which walks a free monoid and concatenates
   it into one String, pushing a single argv word; only on `None` does it fall through to
   `free_monoid_to_vec` and splice. A folded list is monoid-encoded with all-`Str` elements, so
   the concatenating reader succeeds and wins. `gunbc.WitnessBin.Run` escapes this only because
   its `args` is authored as a literal at each call site.

   **Independently attacked:** an outside reviewer attempted to refute this and failed — the
   source matches the reading, and the collapse occurs BEFORE `Command::args`, which rules out a
   downstream shell or SSH join.

   **Why neither obvious repair is correct.** Reordering the arms would splice this case but
   SHRED a modeled `String`, because a String is itself a monoid here (`value_as_host_string`
   reconstructs one from code points): both readings succeed on a monoid of `Str`s, and the
   runtime cannot separate *list of arguments* from *string assembled from pieces*. The
   disambiguator is the DECLARED TYPE, erased at that seam — the same missing
   application-argument typechecking that `target_model` `target_text_carrier_scaffold_note`
   names as the trigger for every type-based construction guarantee at a call boundary.
   Hand-authoring a literal argv in the handler defeats row-derived lowering and leaves the next
   caller silently joined, which is the workaround shape §5 treats as a line-stop.

   **Rung:** this is a §5 fail-open — it fabricates one plausible argument instead of refusing —
   and it is the LANE'S OWN DEFECT CLASS one layer below the lane: Wave 0's witness asserts that
   fragments concatenate within an argument and never across, and the transport violates exactly
   that. It is also a live specimen of the model↔realization fork DESIGN tracks as an open thread.

   **CORRECTION (same day, adversarial review): "blocked on application-argument typechecking"
   was WRONG, and too strong.** The diagnosis above survived independent verification — the
   reviewer traced `build_service_param_env` (which clones call arguments with no conformance
   check), `dispatch_shell`, and `Command::new(&argv[0]).args(&argv[1..])`, confirming the vector
   is already malformed before `Command` sees it, so no later join is available as an alternative
   explanation. The refusal to reorder the branches was also upheld. But the conclusion was not:
   **a correct construction is available now.**

   The counter-evidence is in-tree: `OperationInputValue = InputText | InputTextList` already
   carries tagged list intent, and its realization explicitly converts that tag into a native
   `Value::List` (`free_monoid_to_vec` then `list_value`), so the later flattening splices. Tagged
   intent → native list is therefore proven possible *without* global argument typechecking.

   And the deeper correction: even a fully typechecked system would still not answer the question
   the transport must ask. `List<String>` does not say whether a collection becomes several argv
   words, one JSON argument, a comma-joined option value, or repeated option/value groups. **That
   is a transport ROLE, not a data type**, and the missing fact should be modeled rather than
   inferred from runtime encoding. Application-argument typechecking remains independently
   valuable and is neither necessary nor sufficient here.

   **The construction:** one explicit nominal carrier — `ProcessArgvExpansion { surface: CliSurface }`,
   sealed — with an interpreter branch placed BEFORE the generic string/list heuristic that requires
   the nominal `CliSurface`, iterates its arguments, and pushes exactly one host word per
   `CliArgument`, concatenating that argument's fragments into that one word. It carries `CliSurface`
   rather than `List<String>` precisely so the payload cannot be dynamically ambiguous: a modeled
   string cannot impersonate a list of nominal `CliArgument` records, and `value_as_host_string` is
   then used only on one admitted argument's text, where concatenation IS correct. That is Wave 0's
   theorem enforced at the host edge.

   **Explicitly not to be landed** (each rejected for a stated reason): reversing the two `Variant`
   branches (damages modeled strings); `map(identity)` to coerce a native list, rebuilding the
   emitted argv as a literal, or join-then-shell-split (runtime-representation workarounds, and the
   last reintroduces shell parsing); routing jq through `OperationInputValue` permanently (that
   carrier belongs to declaration-owned operation materialization); or accepting BOTH a raw
   `List<String>` and the explicit carrier (two authorities, with future callers free to pick the
   unsafe one). The raw-list handler signature is REPLACED, not retained for compatibility.

   **Consequence for this lane:** the cutover is blocked at the execution seam *until that carrier
   lands*, which is a build, not a decision. The semantic
   spine is built and resolving; no consumer can move until this is decided, because every
   migrated call site computes its argv rather than authoring it literally — which is precisely
   what the migration asks callers to start doing.

9-bis. **The Class B specimen RECURRED, in code written after it was documented (review 53669,
   2026-08-19).** `jq_invocation_process_plan` used `bind_outcome` and `outcome_accepted` with
   neither in the module's `v2.std.diagnostic` import list. Same mechanism as the
   `cli_long_option_spelling` case one commit earlier, same lane, introduced by the same author
   who had just written that entry.

   **Two things this settles.** First, the mitigation has to run *after every addition*: the
   unimported-reference sweep had been run and was clean, then a new function was added and the
   sweep was not re-run. A one-time audit does not close a class whose defect is invisible.
   Second — and this corrects the review that found it — the stated consequence, "as written this
   file will not resolve", is **false**, and its falsity is the whole point. The file resolves,
   and `stdin_survives_lowering_into_the_process_plan` executes green *through the affected
   function*, because `v2.std.diagnostic` is already in the closure via other imports. A reviewer
   predicting a resolution failure would have been refuted by running it, and might then have
   dismissed a real finding. The correct statement is: the reference is unimported, it binds by
   pool membership, and **nothing in the tree will tell you** — not the resolver, not CI, not a
   green witness.

10. **The spelling-seam extraction is right, but its obvious shape is forbidden — corrected
   2026-08-19.** The static-dependency finding stands: at execution grain the jq fold reaches no
   compiler machinery, but at dependency grain `cli_surface` imports `v2.std.compilers.target_model`,
   whose closure carries target-representation, host-runtime and node-query machinery. The
   property worth having is not *nobody calls translate* but *this fold cannot acquire that
   dependency accidentally*.

   The proposed repair — extract a shared `spell_concrete_syntax_fragments -> Outcome<String>`
   below `target_model`, and have the TargetText serializer wrap its result — **violates an
   operator ruling carried in the file it would edit.** `target_model` `target_text_carrier_scaffold_note`
   (operator, 2026-07-15) places `TargetText` beside `bound_tokens_source_text` so that fold is
   *carrier-native*: the single `String -> TargetText` introduction is `text_atom` applied to one
   grammar-classified token **inside** the fold, the only join is `target_text_seq`, the only exit
   is `render_target_text`, and it states there is deliberately **no** `String -> TargetText` lift
   of an already-composed unit, "that direction is the smuggle the wall closes." Wrapping a
   composed `String` is precisely that lift.

   **The corrected decomposition, and it is the better one anyway.** The shared authority is
   *what is this token's spelling* — `lex_rules_literal_for_class` and `bound_spelling_from_map`,
   both token-grain — **not** *how are spellings concatenated*. Concatenation is carrier-specific
   by construction: emitted source composes through `TargetText`, an argv argument composes into a
   `CliArgument`, and those are different media. So the extraction moves the token type and the two
   token-grain lookups into `v2.std.compilers.concrete_syntax`, `target_model` imports them back
   (re-export is proven to work — see below), and each carrier keeps its own fold. No composed
   `String` is ever lifted, `cli_surface` stops importing `target_model`, and one lookup authority
   still serves both.

   **Re-export is measured, not assumed.** A three-module probe showed a consumer importing a
   symbol from a module that merely imported it resolves and executes. That result alone proves
   nothing, because this resolver can bind by *pool-membership coincidence* (DESIGN's Class B
   finding). The discriminating control settles it: importing the same symbol from a bystander
   module that never saw it **refuses**, located — `name 'ProbeToken' not found in module
   '...bystander'` — even with the defining module in the pool. Qualified-import resolution is
   import-list-driven, so `target_model` importing the seam back keeps its 167 other importers
   untouched.

   **Not performed.** `target_model` is load-bearing and carries the ruling above, so the move is
   its own increment under its own review, not a rider on this one.

11. **A downstream typed verdict may be the redundant lower rung, not a peer** (raised by
   `eager-wren-138`, 2026-08-18). `gunbc.host_effect_realize` `bmcweb_token_extraction_verdict`
   refuses blank stdout as its own decode — the wall that actually closed that fail-open, with
   `jq -e` only ever loudness beside it. If the semantic layer makes *absence-is-a-value*
   unwritable at the decode, that verdict is a second representation of one requirement (§2/§3)
   and must **dissolve into** the decode rather than sit beside it. This is a §4b climb, so its
   discriminating RED stays enrolled against the new decode while the production check is the
   part deleted. Open because it is not yet established whether the requirement is exactly one
   nonempty string at *every* consumer or only at the credential path; the OpenBMC sensor path
   models absence as a legitimate third state (`OpenBmcSensorValueAbsent`), so a blanket
   absence-is-refusal decode would be the state-space collapse this lane exists to remove.

---

## 11. Corrections receipt

Claims asserted during this design and refuted by measurement, kept so they are not re-derived:

- *"The declarative `transport shell` form is fixed-arity and cannot express variadic options."*
  **False** — `ssh.Session.ExecArgv` and `ExecPortableWords` splice `List<String>`.
- *"SSH becomes a bound handler over the same tree."* **Incomplete** — RFC 4254 forces a string
  at that boundary; structure cannot cross it as structure.
- *"`ProcessInvocation` is the general process carrier."* **False** — it is an emit-harness carrier;
  its argument variants are provenance, not CLI roles.
- *"`SensorIntegerValue` has no consumers."* **False** — two live consumers, and its empty-output
  branch is a modeled `OpenBmcSensorValueAbsent`, so an `-e` sweep would have collapsed a correct
  three-state decode into a refusal.
- *"`ExtractRaw` and `SensorIntegerValue` are both latent traps."* **False** for the second; only
  `ExtractRaw` is zero-consumer.
- The plan's census (262 lines / 37 files; 148/51 success/exit_code) is **stale**; measured
  597/129 and 173/69.
- *"`JqProducedNoResult` is a general arm on every jq execution."* **False** — verified against
  `jqlang.org/manual/`: default jq exits 0 regardless of output, and only `--exit-status` yields
  exit 4 for "no valid result was ever produced". Claiming the arm unconditionally would have
  reintroduced the `Absent → Refused` collapse #8454 had just withdrawn.
- *"'Exactly one nonempty string' is a decode the domain can perform."* **Over-claimed** — jq emits
  a stream and `-r` permits newlines inside a raw string, so a collapsed stdout `String` is
  ambiguous. `--raw-output0` exists precisely because of this.
- *"The class sits at mechanically preventable."* **Too coarse** — that is the corpus-wide claim;
  the migrated path can reach structurally guaranteed now via a sealed constructor.
- *"Filters move to `gunbc.bmc_fan_projection`."* **Needed splitting** — only policy-bearing
  programs move; OpenBMC wire decoders stay in `extdeps.bmc`.
- *"`JqProducedNoResult` is an arm beside `JqExitZero`."* **False** — the live `SensorIntegerValue`
  case holds both at once (exit 0, zero results), so termination and output presence are a
  product, not a sum. Written as a sum, the live case is unrepresentable.
- *"Removing `success: Bool` closes the contradiction."* **Insufficient** — an unsealed
  `JqObservation` lets `{ exit: JqExitZero, exit_code: 4 }` be authored, the same contradiction
  under richer names. The carrier must be `sole_constructor`, minted only by the decoder.
- *"The class sits at mechanically preventable because nothing refuses it."* **Internally
  inconsistent** — if nothing refuses it, it is not prevented. Replaced by the subject-grained
  matrix.
- *"jq owns how it spells options AND `CliSyntax` owns canonical spelling."* **Two authorities for
  one decision**, committed inside a note about exactly that.
- *"The carrier probe will hit a named refusal."* **False** — `emit`/`serialize_target` are fixed
  to `Medium<String>` and structured return mismatches fall outside the judged conformance
  population, so a compile-only probe **false-greens**.
- *"The smallest complete vertical includes SSH."* **False** — SSH adds a second language target
  and three independent failure modes; the unit is local-only.

---

## 12. `JqProgram` is public, and the domain still authors jq source

Raised by the `shell → dag` session against the `SensorIntegerValue` cut. The objection: after the
migration, `extdeps.bmc.openbmc_fan_control` no longer authors argv — but it *does* author

```
data openbmc_sensor_integer_program: String =
  "if (.data | type) == \"number\" then (.data | round) else empty end"
```

so the workflow moved from authoring shell to authoring jq. `JqProgram { source: NonEmptyStr }`
is a public constructor, so every domain module may do the same.

**The objection is right about the residue and should not be argued down.** What the cut
established is narrower than "the domain stopped authoring foreign syntax": it stopped authoring
*argv*. Measured at the cut, OpenBMC no longer names `-r`, `--raw-output`, an option spelling, an
argument position, argv[0], or the stdin routing — all of those are now derived from jq's cited
rows, and that is the property the two-level derivation buys. The jq *program* is untouched by it.

**The proposed remedy — keep `OpenBmcSensorIntegerProjection { content }` public and have the
handler privately select a sealed program identity — is refused as stated, on §2.** It does not
decompose the program; it relocates the same string behind an indirection and adds a registry to
maintain. Ask where the text lives afterwards. If the handler is OpenBMC's, the jq source is still
in the OpenBMC layer and only a wrapper was added. If the handler is jq's, then `extdeps.tools.jq`
knows what a BMC sensor reading is — a layer inversion, jq depending upward on its consumer's
domain. Neither position is better than the row that exists, and the second is worse. A sealed
identity would buy something only if the identity were *derived*, which is the real answer:

**The terminal shape is that the program is not authored at all.** `.data`, "is it a number",
"round it", "otherwise nothing" are a modeled JSON projection — a path, a type guard, a rounding
policy, and an absence arm — of which a jq program is ONE realization, exactly as an argv vector
is one serialization of a `CliSurface`. Under that model the domain declares the projection, jq's
module derives the program text from it the way it already derives option spellings, and a second
realization (a native JSON fold with no process at all) becomes possible for free. That is the same
move the lane already made one layer out, applied one layer in — and it is what stops this work
generalizing into a thousand bespoke jq programs where it found a thousand bespoke argv lines.

**Rung, honestly.** Domain-authored jq source is **mitigatable**: nothing prevents it, and the only
things standing between it and an arbitrary computed program are that the row is `data` rather than
a function — so it cannot vary per call — and review. Not a wall, and it should not be reported as
one.

**Dissolution trigger:** a modeled JSON projection carrier with jq as a derived realization. Until
it lands, `JqProgram` stays public and this section is the reason, not an oversight. This is
deliberately NOT scoped into the current cut: modeling a JSON projection algebra is its own
vertical, and bundling it would repeat the Wave 1A mistake of selecting the richest thing first.


---

## 13. The observation contract travels with the plan

Raised as "`JqAdmittedProcessPlan` cannot be decoded" by the `shell → dag` session. Verified
against the code and **worse than raised** — it was not a missing convenience, it was a live latent
defect in the cut itself.

The sensor decoder read `exit_code`, `stdout` and `stderr` directly and mapped every nonzero exit to
a refusal. That is correct under `JqProgramExit` and silently wrong under `JqLastResultExit`, where
jq documents exit **1** as "the last output was false or null" — a legitimate *value* — and exit
**4** as "no valid result was ever produced" — *absence*. Nothing connected the decoder to the
policy that produced the argv, so changing `exit_policy` in the invocation would have left the
decoder confidently answering the previous contract. That is §5's own tell: the declaration is
edited while the realization goes on lying, and the model's own witnesses stay green because they
never varied the policy.

**Repair.** `JqAdmittedProcessPlan` carries `exit_policy`, so the plan is self-describing. One
authority, `jq_classify_observation`, reads an exit code *against* the policy that asked for it and
returns `JqOutputPresent | JqOutputAbsent | JqExecutionRefused`. The domain matches those arms and
decides only what OpenBMC alone can decide — whether a present projection parses as an integer. No
domain module reads a jq exit code any more.

**Discriminating evidence, and why it discriminates.** Five assertions hold one observation fixed
and vary only the policy:

```
exit 1, stdout "false"   under JqProgramExit     -> Refused
exit 1, stdout "false"   under JqLastResultExit  -> Present     <- the policy-blind decoder fails here
exit 4, stdout ""        under JqLastResultExit  -> Absent
exit 2, stderr "usage"   under BOTH              -> Refused
exit 0, stdout "   "     under JqProgramExit     -> Absent
```

The second row is the discriminator: a decoder reading the exit code without the policy must answer
`Refused` for exit 1 under both policies, so it cannot make rows one and two true at once. The
fourth row is the control that keeps the law from overfitting — the policy changes how *some* codes
read, not whether jq can fail.

**This also closes the point `eager-wren-138` reached from the other direction.** That session
proposed sweeping `-e` across jq sites as hardening; the counter was that `-e` reports on the *last
output*, so it converts legitimate `false`/`null` into failure. Both facts are now modeled rather
than argued: selecting `JqLastResultExit` changes the *decode*, not just the flag, and absence
remains a state distinct from refusal on both arms.

---

## 14. Verification scoped to the consumers I already knew about

Recorded because it is the third instance of one failure mode in this lane's own work, and the
first two were caught by other people.

Changing `openbmc_sensor_integer_projection_result`'s signature broke two enrolled witnesses
(`failed_sensor_projection_cannot_become_absent`, `successful_empty_sensor_projection_is_absent`)
that call the decoder directly. I did not find them, because I had selected the witnesses to run by
asking which files import `openbmc_fan_control.dag` **and then hand-picking three of them** — the
verification denominator was a list I wrote, not a list the tree produced. The break surfaced only
when the resolve failed on an unrelated run.

The same shape produced the two earlier defects: an unimported-reference sweep run *before* the
last addition rather than after it, and a grep whose zero result was accepted without a control
that must hit. In each case the mechanism was sound and the **denominator** was authored.

**A third instance, one level up, caught by CI after the above was written.** The rule below fixed
the witness denominator and was still not enough: I minted `type ExpectedOutcome` in the jq witness,
colliding with `gunbc.output_policy`'s load-bearing carrier of the same name, and the failure landed
in `output_policy_witness_test.dag` — **a file that references none of my symbols and that this
branch never touched.** Bare-reference resolution is corpus-global, so a new top-level name can red
a file no per-file closure of mine would ever load. Every local run passed; the whole-corpus strict
resolve is what caught it.

Two rules follow, and both are mechanical because the authored version keeps failing:

```
# every top-level name the branch ADDS, from the diff -- not from memory
git diff main...HEAD -- '*.dag' | grep "^+"   | grep -oE "^\+\s*(type|fn|data)\s+[A-Za-z_][A-Za-z0-9_]*" | awk '{print $NF}' | sort -u
# each one must have exactly ONE declaration corpus-wide

# and run what CI runs, before pushing, not after it fails
claim_executor --required-floor --source-root dag --source-root src/v2
```

The second is the one that matters: **a per-file run and the floor are different universes**, and
only the floor is the acceptance path. Reporting a per-file green as though it were the floor is
rung inflation against a declared boundary (§4b) — the same error as citing the strongest path while
another stays silent, committed against my own change.

**The rule adopted for the rest of the lane:** verification enumerates test functions from the
FILE, never from memory —

```
grep "^test fn" <witness> | sed 's/test fn \([a-z_0-9]*\).*/\1/'
```

— and every function in every witness file touching a changed signature runs, not a chosen subset.
This is the same principle §5 states for oracles, applied to coverage: a population I author is not
an observation of the population that exists.

**What the break itself demonstrates, and it is the argument for the cut.** Both witnesses were
re-enrolled unchanged in *requirement* — a refusal must not decay into absence, an empty successful
projection must not decay into a refusal — while their *carrier* moved from a `success: Bool` triple
to the typed jq outcome. The migration also made a third requirement stateable that the old triple
could not express: that a present numeric projection is an observation. That is a replacement
migration behaving correctly (§3): deletion surfaced the load, the load was dispositioned as
re-enrolled evidence rather than restored by reflex, and the new representation was strictly more
expressive than the one it replaced.

---

## 15. Wave 1B cutover receipt

Executed on BuildBuddy against the branch tree. Counts are from enumerating `^test fn` in each
file, not from a chosen subset (§14).

```
extdeps.bmc.openbmc_fan_control                 SensorIntegerValue operation DELETED
  consumers rewired                             2 (openbmc_sensor_value, openbmc_sensor_threshold_result)
  both now route through                        openbmc_sensor_integer_projection  (one shared fn, not two call sites)
  argv authored by the domain after cutover     0 -- no -r, no --raw-output, no argv position, no argv[0]
  residual foreign syntax authored              1 jq program row (S12, mitigatable, trigger recorded)

test/claim/bmc_typed_operations_witness         29 / 29 pass
  of which migrated onto the typed jq outcome    2 (requirement unchanged, carrier replaced)
  of which newly stateable and added             1 (present numeric projection is an observation)
test/claim/jq_invocation_lowering_witness        9 / 9 pass  (lowering)
                                                 5 / 5 pass  (S13 exit-policy classification)
test/claim/bmc_fan_converge_witness              duty_curve pass
test/manual/process_argv_expansion_receipt       case 4 pass -- real jq, exit 0 reachable only
                                                 with two argv words, through the production handler
```

**What this receipt does NOT establish**, stated because the table above reads stronger than the
lane's actual position:

- The argv splice defect is repaired at **one seam**, not closed as a class. 21 operations still
  splice a declared `List` parameter through the guessing path.
- The other three `openbmc.JsonProjection` operations are unmigrated, and two of them
  (`ObjectMapperServiceCount`, `ObjectMapperServiceAt`) need `--argjson` lowering, which is unbuilt.
- No negative falsifier exists yet proving a REST path reaches none of these carriers.
- **The wet receipt does not execute in CI, and this is a declared gap rather than an enrollment.**
  The hermetic floor refuses `jq.Process.RunWithStdin` (no `mock_response`) and mocking would
  defeat the assertion, so the file is named in `cli_run.rs` `floor_prepared_subject_exclusions`.
  No wet lane exists to host it: the falsifier and wet batches died with the floor cut, and
  `v2.workflow.required_floor` deliberately defers wet lanes until the question can be "asked
  against a live consumer". So the claim sits at **UNEXECUTED-IN-CI**, evidenced only by the
  recorded run above and reproducible locally with:

  ```
  claim_batch --wet --source-root dag --source-root src/v2     --entry dag/test/manual/process_argv_expansion_receipt_test.dag     --functions case4_expansion_carrier_splices
  ```

  **Re-enrollment trigger:** a wet lane with a live consumer exists again.

  The first attempt at this admission added rows to `gunbc.ci_layer_roots`
  (`witness_exclusion_frontier` + `bin_witness_wet_entries`) and a commit message asserting they
  would take effect. They did not — `run_required_floor` consults the Rust list and nothing else,
  and the CI receipt was an unchanged `modules_excluded=2`. Both rows were reverted rather than
  left standing: a row in a roster with no live consumer is specification-without-execution, and
  the enrollment half would have been worse than useless, since it would have *claimed* an
  executing consumer for a witness nothing runs. Recorded because the mistake is instructive —
  I read a neighbouring row's shape as the mechanism instead of tracing the mechanism's caller.
- The wet receipt run reports `[expectation-frontier] 1 site(s), 1 dispatch(es) undeclared:
  jq.Process.RunWithStdin=1`. That is the new handler's dispatch not being declared to the
  expectation registry. It is left standing rather than silenced because it is exactly the shape §5
  asks for — typed, located, counted, and visible — but it IS an open item, and a reader should not
  infer from a `PASS` line that nothing else was reported.
- The 5 classification assertions and the wet receipt executed in separate runs from the 29+9; a
  single run of all 43 exceeded the 45-minute remote timeout, because every `gunbc run` pays a
  whole-corpus typecheck. That is a cost-shape observation about the harness, not evidence about
  the code, and it is recorded rather than left as an unexplained split.

## 16. The negative falsifier: the REST/CLI boundary becomes a row that reds

Every prior section of this document asserts, in prose, that the REST path constructs no
process invocation. Prose cannot be contradicted by the tree. This section records the
mechanism that can.

**The question already had an authority, so no second mechanism was minted.**
`v2.lens.realization_vocabulary_containment` answers "does module X reach construction
vocabulary from set V" and has answered it since the TypeScript-only ratchet. The obstacle
was not that the question was unmodeled but that `V` was a literal: `scan_facts_for_leaks_under`
threads the *importer-path* axis as a parameter while `module_is_target_ast_vocab` was welded
into the predicate. Building a second lens for a second `V` would have been precisely the §3
duplication this lens exists to detect, so the vocabulary axis was opened instead — the §2
horizontal move, one axis rather than N copies.

What that cost, exactly:

- `RealizationVocabularySet` (name, modules, module_prefixes) and `module_is_in_vocab`.
- `target_ast_vocabulary()` derives from the existing `target_ast_vocab_modules` /
  `target_ast_vocab_module_prefixes` rows rather than replacing them — those rows are consumed
  directly by `gunbc.realization_vocab_confinement_census`, which has live claims against them.
- `is_vocab_leak_in` / `scan_facts_for_leaks_in` / `vocab_leak_count_in` / `vocab_leak_count_live_in`,
  taking vocabulary and exempt-edge population as arguments. Every pre-existing entry point
  delegates to these with the target-AST set, so no behavior moved.
- The exempt population is a **parameter**, not a global roster read, so "this vocabulary has
  zero admitted exceptions" is a stated fact rather than an accident of the target-AST roster
  happening to name no CLI module.

**Two sites were left target-AST-only, deliberately.**
`realization_vocab_leak_candidate_paths_from_facts` and
`realization_vocab_live_leak_edges_from_facts` feed the grandfathered-roster staleness check,
and every row in that roster is target-AST debt by construction — `RealizationVocabDebtClass`
has no other inhabitant. A second vocabulary reaches the lens with an empty exempt population,
so it has no roster for those projections to be stale against, and parameterizing them now
would produce a staleness answer about a population that does not exist. The trigger is
recorded in-file: the first non-empty exempt roster for a second vocabulary.

**The subject is not an empty universe** — the usual way a negative claim turns vacuous. The
scanned population (`dag/extdeps/bmc`, `dag/extdeps/transports`) *contains* a module that
legitimately reaches CLI vocabulary: `extdeps.bmc.openbmc_fan_control`, the module this very
lane routes through jq, sitting in the same directory as `extdeps.bmc.redfish`, which does not.
So the scan discriminates *within* the population, and the RED control is live corpus data
rather than a planted fixture: withdraw the one admitted edge and the count must become 1.
Without that assertion the positive result is indistinguishable from a scan that read nothing,
which is the empty-observation narrow DESIGN names — ⊥-as-answer conflated with ⊥-as-ignorance.

The admitted edge is named at exact `(importer_path, vocab_module)` grain rather than by
category or path prefix, so a *second* jq-reaching module anywhere in the scanned roots reds
instead of being absorbed by a pattern broad enough to cover it.

**What executes, and what does not.** The witness is floor-discovered — it is neither
`long/`-homed nor in `floor_prepared_subject_exclusions` — and its scan is a live read of the
two named directories, so this is corpus data rather than a fixture. It is **not** whole-corpus
coverage: a module outside those roots reaching CLI vocabulary is not seen, and no green here
may be read as saying otherwise. The whole-corpus half of this lens is enrolled on a cadence
that does not currently run, which is a fact about that cadence, not about this witness. Both
halves are stated because a scoped green presented as a general one is the rung inflation §4b
calls worse than sitting low.

**Where this goes, not scoped here.** The boundary in #8535 is a paragraph today. If this
generalization holds, the natural successor is that the paragraph becomes a row that reds —
which changes what the lens is *for*: not one witness for one lane, but the mechanism by which
a named architectural boundary is enforceable at all. That is a separate change and is
deliberately not attempted in this one.
