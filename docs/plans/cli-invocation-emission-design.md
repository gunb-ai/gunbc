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

## 9. Rung, by exact subject

An earlier revision said "mechanically preventable, because a caller can still construct a raw
`List<String>` and nothing yet refuses it." That sentence is internally inconsistent: **if nothing
refuses it, it is not mechanically prevented.** It is measured or mitigated, not prevented. It
also *understated* what the migrated path can reach. One number for several populations was the
error.

| exact subject | honest rung after the first vertical | mechanism |
|---|---|---|
| forging a `CliInvocationTree` / `CliSurface` / `JqObservation` | **structurally impossible now** | `sole_constructor`; only the row emitter, serializer and decoder may mint |
| omitting `--exit-status` from an accepted truthiness invocation | **structurally impossible within the new path** | exhaustive semantic→option selection; zero-or-many rows refuse; no mint on miss |
| hand-authoring flags inside a migrated route | **structurally impossible after cutover** | handler takes a semantic invocation, emits internally, old argv site deleted |
| reintroducing the exact old operation transport | mechanically preventable | structural deletion witness |
| writing another raw jq argv elsewhere | **still writable** | unrelated raw shell/process routes remain |
| arbitrary raw argv corpus-wide | outside this vertical | later process/transport confinement |

So: **the first migrated jq path can be structural now; the repository-wide raw-jq class stays
open; the repository-wide raw-argv class is a later migration.**

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

9. **A downstream typed verdict may be the redundant lower rung, not a peer** (raised by
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
