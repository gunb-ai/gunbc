# CLI invocation emission — the tree is the authority, argv is emitted

**Status: DESIGN NOTE. No code lands from this note yet.** Design-note-first, per the
hollow-alias precedent. The first vertical is specified here; nothing is built until the
note is accepted.

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

### 4b. Observation, decoded **policy-indexed** — not once

An earlier revision of this section put `JqProducedNoResult` as a general arm on every jq
execution. That was wrong, and wrong in the exact direction #8454 had just corrected: it claims a
distinction the selected exit mode cannot supply.

Verified against `jqlang.org/manual/`:

```
default             exit 0 when the program runs successfully, REGARDLESS of output —
                    emitting nothing is still exit 0
--exit-status / -e  exit 0  last output value neither false nor null
                    exit 1  last output value false or null
                    exit 4  no valid result was ever produced
halt_error          program-chosen code, default 5
usage/system 2      compile error 3
```

So "no result was produced" is **observable only under `JqLastResultExit`**. Under
`JqProgramExit`, exit zero means the program completed and says nothing about whether the output
stream was empty. Decoding is therefore indexed by the policy the caller selected:

```
type JqExitPolicy = JqProgramExit | JqLastResultExit

type JqExecution
  = JqProgramCompleted      { stdout, stderr, exit_code }   // no claim about results produced
  | JqLastResultFalseOrNull { stdout, stderr }              // JqLastResultExit only
  | JqProducedNoResult      { stdout, stderr }              // JqLastResultExit only
  | JqExecutionRefused      { exit_code: Int, stderr }
```

```
JqProgramExit    + exit 0  -> JqProgramCompleted
JqLastResultExit + exit 1  -> JqLastResultFalseOrNull
JqLastResultExit + exit 4  -> JqProducedNoResult
```

The raw `exit_code` is preserved alongside the decoded arm, so decoding known meanings never
discards the observation. Spawn failure and signal termination belong to the **process layer**,
not to jq exit decoding — jq cannot report an exit code it never reached.

**Why this matters concretely.** `SensorIntegerValue` runs under `JqProgramExit`. Its
`OpenBmcSensorValueAbsent` fact is derived by the OpenBMC domain decoder from *successful, empty
stdout over that exact program* — **not** from a jq exit code. Putting a general
`JqProducedNoResult` arm on jq would invite exactly the `Absent → Refused` collapse that the
blanket `-e` sweep was withdrawn for.

**A stdout `String` cannot generically prove "exactly one nonempty raw string".** jq emits a
*stream* of results; under `--raw-output` a raw string may itself contain newlines, which is why
`--raw-output0` exists ("Like `-r` but jq will print NUL instead of newline after each output…
When the output value contains NUL, jq exits with non-zero code"). So `foo\nbar\n` is ambiguous
between one string containing a newline and two results, and once the process boundary has
collapsed stdout into one `String` no generic decoder can separate them.

This does not block the first vertical — an integer has an unambiguous textual grammar, empty is
distinguishable from one integer, and multiple numeric results produce multi-line text a
`parse_int` refuses. It does mean **this note promises no generic
`decode_exactly_one_nonempty_string`.** For the credential path later, one of three honest
framings must land first: a single JSON envelope parsed as JSON; `--raw-output0` once jq
version/capability is modeled; or a program collecting its result into a one-result container the
domain decoder checks. Until then a decoder may enforce "nonempty observed stdout" and must not
claim "exactly one jq string result".

**No `success: Bool`.** The seed derives it as exactly `exit_code == 0`, so the pair carries one
fact twice and makes a contradiction writable — a caller can pass `success: true` beside a nonzero
code. `extdeps.git.git` `ls_remote_reports_termination_not_success_note` already establishes this
and names itself the first consumer of the correction. 173 operations still carry the Bool.

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
| `extdeps.tools.jq` | what raw output means, what exit-status mode means, that `--argjson` takes a name and a JSON value, which options repeat, how jq spells them |
| shared `CliSyntax` | option/value structure, repeated options, operand ordering, separators, canonical short/long spelling, emission to argument tokens |
| `gunbc.bmc_fan_projection` | the `TEMP_SOC` policy, the fan curve, controller cardinality, the desired topology |
| `extdeps.bmc.*` | programs projecting a documented OpenBMC response or config field |
| realization | local vs SSH vs another handler |
| **no caller** | `-e`, `-r`, `--argjson`, `--`, or their positions |

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
5. The CLI inverse consumes `List<CliArgument>`, not shell text. Shell text is parsed by the shell
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

**Canonical first vertical: `SensorIntegerValue`.** Chosen because it has two live consumers; its
three-way domain contract is already explicit and witnessed; it exercises stdin as a *process
channel* rather than an argument; it proves `JqProgramExit` must emit **no** `--exit-status`; it
proves empty output can be a valid domain observation rather than a transport failure; its integer
output sidesteps the raw-string framing problem above; its program is small enough that the
architecture stays visible; and deleting its old operation is an exact, countable cut.

Acceptance population for it:

```
{"data": 41.6}      -> Observed 42        {"data": "41"}     -> Absent
{"data": 41}        -> Observed 41        {"data": null}     -> Absent
{}                  -> Absent             invalid JSON       -> Refused
two numeric inputs  -> Refused, never first-pick              jq nonzero -> Refused
```

Emission side, proven separately from semantics:

```
JqRawOutput selects the raw-output option identity
that identity selects its canonical spelling
JqProgramExit emits NO exit-status option
the jq program is exactly one argument
stdin content appears NOWHERE in argv
argument order is stable
```

**Wave order after the proof.** Slice A `SensorIntegerValue` (above). Slice B
`ObjectMapperServiceCount` + `ObjectMapperServiceAt` together — adds `JqLastResultExit`,
`JqJsonBinding`, the two-argument `--argjson name JSON-text` row, canonical JSON serialization of
`index`. Slice C `ProjectFanConfig` — adds file input, JSON output, the `desired` binding, and
relocation of the policy-bearing program to `gunbc.bmc_fan_projection`. Slice D one remote
file-input scalar query — adds the password-SSH handler, arguments embedded through the shell
grammar, and an explicit stdin-input refusal for the `sshpass -d 0` channel. Each slice deletes
its old operations in the same cut. After D the remainder is repetitive migration, not
architecture.

**Do not make the first vertical solve SSH.** Local exemplar and transport composition are
separate proofs.

**First vertical (smallest complete):**

1. `JqInvocation` and its semantic axes.
2. `CliInvocationTree`.
3. Shared CLI grammar rows + jq's cited option rows.
4. Emission to `List<PosixArgument>`.
5. Local resolution into `extdeps.posix` `Command`.
6. A process-input realization for jq stdin, separate from argv.
7. The SSH handler: emitted arguments → shell simple-command tree → shell text → RFC 4254 string.
8. One live jq population cut over, with its prior argv authoring site deleted.

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

## 9. Rung, honestly

The rung is **two different rungs for two different subjects**, and stating one number for both
was too coarse.

**For the migrated jq path: structurally guaranteed is reachable now.** If
`AdmittedCliOptionUse` is a `sole_constructor` record whose single mint is `admit_callers`-sealed
to the jq semantic-to-syntax projection, then no domain module can form an option use at all —
the same construction wall that already closed `TransportScript`, which needed both the record
seal and the cast judgement in `04_infer` `sole_constructor_construction_diags`. A migrated
caller cannot author `-e`, omit a required option, or mis-arity `--argjson`, because it holds no
constructor for any of them.

**For the corpus at large: mechanically preventable at best.** A caller can still write a raw
`List<String>` in any of the other 575 argv lines, and nothing refuses it. That is the weaker
claim, and it does not climb until the population is migrated and a construction guard rejects
raw argv authoring outside a declared foreign/bootstrap escape.

The ceiling for the *nominal-carrier* half remains **structurally impossible**, and its named
trigger is application-argument typechecking in `04_infer`. `v2.std.compilers.target_model` `target_text_carrier_scaffold_note`
records — proven by execution — that neither the v1 seed nor v2 `04_infer` typechecks
function-application argument types, so a raw `String` passed where a nominal carrier is declared
compiles clean. **A nominal `CliArgument` newtype alone therefore makes nothing unwritable today** — which is
exactly why the wall must be the sealed constructor, not the type name.
Until that frontier closes, this design's guarantee rests on executing witnesses, and the
construction wall rejecting raw argv authoring — except an explicitly modeled escape population
at genuine foreign/bootstrap boundaries — is a later rung.

Claiming otherwise would be rung inflation.

---

## 10. Open questions

1. **Carrier parameterization.** The terminal shape is conceptually
   `emit(tree, text_target) -> Medium<TargetText>` beside `emit(tree, cli_target) -> Medium<CliSurface>`.
   `Medium<R>` is already generic; the text lock is in `TargetModel`, `serialize_target` and `emit`.
   **Whether that generic spelling compiles is unproven and needs an executing probe.**
   Not-negotiable either way: do not add a fake textual encoding to avoid changing the seam.
   Space-separated text reintroduces quoting where none exists; NUL-delimited text is a wire
   encoding masquerading as the target artifact.
2. **`sshpass -d 0` and stdin.** The password occupies fd 0. A remote stdin-fed jq would collide.
   Latent, not live — all seventeen remote sites use file operands. Must be a typed refusal, not
   an accidental limitation. Options: remote file-input only; move password delivery to another
   descriptor; or a framed remote receiver.
3. **Portable-word allowlist.** `gunbc.typed_argv_exec` bounds what may cross SSH. jq programs
   contain spaces, pipes, brackets, parentheses and quotes — outside that alphabet. The shell leg
   must own canonical shell-word serialization with injection controls, or refuse.
4. Which of the 315 computed-head argv lines hide further wrappers.
5. **The remote jq rows may already be broken — live, unverified.** The seventeen
   `openbmc_password_ssh_transport` operations place the jq filter after the SSH destination as
   though its local argv boundary survived remotely. It does not: the local `ssh` client joins its
   command words with spaces into RFC 4254's single `command` string, and OpenSSH's server runs
   that through the user's login shell, which re-parses it. Those filters contain spaces, pipes,
   brackets, parentheses and double quotes — `[.zones[].pids[] | select(.name == "TEMP_SOC" …)] |
   length` would reach the remote shell with `|` as a pipe operator and the quotes consumed.
   **This is a transport-correctness risk, not a proven defect** — no executed control has been
   run, and something upstream may be quoting. It must be settled by execution before Slice D, and
   it is an argument for the design rather than a consequence of it: the current representation
   cannot even express the question.
6. The SSH slice needs a discriminating program containing spaces, a pipe, quotes, brackets and a
   value containing a single quote, and must reuse the grammar-owned quoting the effect-plan Bash
   path already demonstrates rather than retaining `shell_quote`.
7. Reverse ingestion — reading argv back through the same rows — is the terminal second reading
   and is **not a prerequisite for the first emitter cut**.

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
