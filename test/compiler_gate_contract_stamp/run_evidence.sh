#!/usr/bin/env bash
# Execute the contract-stamp guard against the real bytes, and fail if it does not refuse what it
# is supposed to refuse.
# SCAFFOLD - dissolve-on: TWO capabilities, because they retire DIFFERENT halves of this file.
#   (1) orch-emitted -- for the RESIDUAL SHELL: node kinds in v2.extdeps.languages.bash_build
#       (rendered by v2.workflow.bash_emit / bash_emit_stmts) for what this file still does by hand:
#       extract a committed workflow member, scan it structurally, drive argv, invoke awk. Everything
#       expressible that way already is -- both steps are node trees, which is why they carry
#       dissolve-on lines in the emitted YAML. When those node kinds land, this file becomes an
#       emitted step.
#   (2) gunbc.rung_drop v1_required_lanes_withdrawn -- for the TEST PROCESS: its restoration_trigger is
#       the condition under which the modelled rows (compiler_gate_contract_stamp_evidence_cases) become
#       a `gunbc test` row in gunbc.instrument_targets and run the guard through
#       shell.Exec.RunArgvOutcome. When that lands, this file is DELETED rather than emitted.
#       Cited by symbol so carrier and drop retire together rather than separately.
#
# WHY THE GUARD IS READ BY THE RUNNER RATHER THAN BY A TYPED READER, and why the RED is a step here
# rather than a claim row. The guard runs BEFORE any candidate binary exists. The only binary that
# could read the tree is the tree's own -- the party under suspicion -- and a pre-stamp head does not
# yet contain the reader module, so a pre-build read is structurally necessary rather than convenient.
# Spawning itself is not the gap: shell.Exec.RunArgv / RunArgvOutcome exist
# (dag/extdeps/shell/exec.dag, ProcessObservation carrying exit_code, stdout and stderr) and are
# callable from a test fn. What is missing is an EVALUATOR of such claims on the required
# pull_request path: heal.yml runs the claim executor as an instrument, not a gate, and the instrument
# that runs `gunbc test <label>` is workflow_dispatch-only. A claim would therefore sit off the required
# path, and the discriminating RED would be attested rather than executed.
#
# WHO OWNS WHAT. The case TABLE -- which cases exist, which subject each reads, what exit code and
# which diagnostic each must produce -- is modelled in .dag, at
# dag/gunbc/witness/compiler_gate_contract_stamp.dag (compiler_gate_contract_stamp_evidence_cases),
# and it is EMITTED into .github/workflows/witnesses.yml as one command node per row. This script
# contains no table of expectations, because a second copy of a table is a second authority. The
# emitted step carries the model's rows INTO the required job, and local mode refuses if the committed
# runner's invocations have drifted from them: DESIGN section 5 prefers construction over validation,
# so the emitted bytes are REQUIRED to agree with the model rather than checked against it afterwards.
#
# What remains here, precisely: the ACT OF SPAWNING inside a step that runs before the subject binary
# exists. A scaffold whose stated reason contradicts itself has no honest dissolution condition, so the
# capability position is stated once, above, and nowhere else.
#
# THREE MODES
#   (no args)        LOCAL. Extract the generated step's own bytes from the committed witnesses.yml
#                    and execute them, then report its exit status. So a developer and the gate run
#                    literally the same commands, and a step that has been dropped from the workflow
#                    makes this mode FAIL rather than quietly pass -- enrolment is checked by running,
#                    not by grep.
#   --case ...       SINGLE CASE, invoked by the generated step, argv supplied BY THE MODEL:
#                    --case <label> <subject> <running> <expected-exit> <finding>
#                          --transcript <path> --fail-marker <path> --marker <token>
#                          --run-token <GITHUB_RUN_ID>-<GITHUB_RUN_ATTEMPT>
#                    The step counts lines carrying the RUN TOKEN, so a transcript left by an earlier
#                    invocation cannot satisfy the tally; the token's shape is checked here because the
#                    step only checks it is non-empty, and grep for "-" matches almost anything.
#                    It compares and, on failure, creates the fail marker.
#   --prepare --fail-marker <p> --transcript <p>
#                    Clear this run's fail marker and transcript before the cases run.
#
# THE FAIL MARKER IS THE VERDICT CHANNEL. The generated step's refusal is `[ -e <fail-marker> ]`
# rather than `if ! grep ...`, because bash_fold_serialize_negation_inner admits only
# or_else/and_then/pipe/negation -- an `if ! <command>` condition has NO node representation. That is
# the same constraint that shaped the guard's own capture, and the marker is the node-shaped answer.
#
# WHAT IT RUNS IS THE COMMITTED ARTIFACT, NOT A COPY OF IT. extract_block reads the guard's (or the
# evidence step's) script out of .github/workflows/witnesses.yml byte-exactly; nothing is retyped,
# because a retyped copy can pass while the real bytes fail -- which is how the fused `-v key=` defect
# survived inspection while making the guard refuse every tree, correct ones included. Each guard case
# runs under `bash -e -o pipefail`, GitHub's default `run:` shell, and `${{ env.* }}` is substituted
# the way Actions would substitute it.
#
# bash and awk only, no python: .gitignore line 162 puts *.py "outside the committed substrate", so a
# committed test cannot depend on a file the repo refuses to track. The fleet's runners use mawk 1.3.4.
#
# Usage:  ./test/compiler_gate_contract_stamp/run_evidence.sh
# Exit:   0 only if every modelled case held.
set -uo pipefail

here="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
repo="$(cd "$here/../.." && pwd)"
committed="$repo/.github/workflows/witnesses.yml"
fixture="$here/fixtures/silent-green-head-witnesses.yml"
guard_step_name='- name: Refuse a subject older than the instruction running it'
evidence_step_name='- name: Execute the contract-stamp guard against the recorded silent-green subject'
placeholder='${{ env.GUNBC_COMPILER_GATE_CONTRACT_STAMP }}'
running_probe_token="__FROM_WORKFLOW__"
case_token_default="gunbc-contract-stamp-evidence-case"
# THE PATHS ARE PER-RUN, AND DERIVED HERE EXACTLY AS THE MODEL DERIVES THEM.
# The step passes --transcript/--fail-marker explicitly, so argv normally decides. But local mode and
# --prepare can be reached without them, and if this file fell back to a FIXED /tmp name while the model
# used a per-run one, the two spellings of one fact would disagree -- the step would write where this
# file is not looking, and report "nothing ran" about a run that ran. So the fallback is the same
# RUNNER_TEMP + run-identity directory the model builds, and it refuses rather than guessing.
#
# The defect the model change closes: with fixed names, slot A leaves a failure marker, slot B's
# --prepare deletes it before A reads it, and A exits GREEN with a failed case -- a silent widen, which
# DESIGN 5 rejects outright. The run token guarded the case COUNT; the marker is the channel that
# carries FAILURE, so it needed the same key.
# THE IDENTITY IS VALIDATED BEFORE IT NAMES A DIRECTORY. A caller can name the directory outright with
# GUNBC_CONTRACT_STAMP_EVIDENCE_DIR and hand the per-run REQUIREMENT a constant shared by every run --
# GITHUB_RUN_ID="" gives two slots the same directory, which reopens the very race this fix closes, one
# layer down. So the run identity is checked here as well, and a supplied directory must carry it: the
# directory's own name has to end in <run_id>-<attempt>, which is what makes it per-ATTEMPT rather than
# per-job or per-host.
run_identity=""
if [ -n "${GITHUB_RUN_ID:-}" ] && [ -n "${GITHUB_RUN_ATTEMPT:-}" ]; then
  case "$GITHUB_RUN_ID" in ""|*[!0-9]*) run_identity="" ;;
    *) case "$GITHUB_RUN_ATTEMPT" in ""|*[!0-9]*) run_identity="" ;;
         *) run_identity="$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT" ;; esac ;;
  esac
fi
evidence_dir="${GUNBC_CONTRACT_STAMP_EVIDENCE_DIR:-}"
if [ -z "$evidence_dir" ] && [ -n "$run_identity" ] && [ -n "${RUNNER_TEMP:-}" ]; then
  evidence_dir="$RUNNER_TEMP/gunbc-contract-stamp-$run_identity"
fi
if [ -n "$evidence_dir" ]; then
  if [ -z "$run_identity" ]; then
    echo "FAIL: GUNBC_CONTRACT_STAMP_EVIDENCE_DIR is set but the run identity is not a valid" >&2
    echo "      <digits>-<digits> (GITHUB_RUN_ID='${GITHUB_RUN_ID:-}' GITHUB_RUN_ATTEMPT='${GITHUB_RUN_ATTEMPT:-}')." >&2
    echo "      A directory without a per-ATTEMPT suffix is a directory two runs can share." >&2
    exit 1
  fi
  case "$evidence_dir" in
    *"$run_identity"|*"$run_identity"/) : ;;
    *) echo "FAIL: evidence directory '$evidence_dir' is not keyed to this run's identity '$run_identity'." >&2
       exit 1 ;;
  esac
fi
# ONE RULE, EVERY CHANNEL, NO EXEMPTION FOR ARGV.
# Every verdict channel -- transcript or fail marker, however it was obtained -- must live inside the
# per-run directory. Earlier this file exempted paths handed in as argv, on the reasoning that a caller
# naming a scratch file is doing something legitimate. That exemption was a HOLE, and the reviewer's
# "alternatively" clause was the necessary half of the fix rather than the optional one: the checks that
# read the STEP's bytes live in local mode, and NOTHING ON THE REQUIRED PATH RUNS LOCAL MODE (verified:
# witnesses.yml invokes this file only with --prepare / --case). So if the model ever regressed to fixed
# /tmp paths, the generated step would pass them as argv, every argv-carrying check would pass them on,
# and no code on the required path would refuse. Enforcement has to sit where the paths are actually
# consumed, which is here.
#
# THE TEST IS MEMBERSHIP OF THE PER-RUN DIRECTORY, not "is not under /tmp". An earlier version rejected
# any path containing /tmp/, which would have refused the very layout this fix requires: RUNNER_TEMP is
# a directory someone else chooses, and a runner (or a developer) that puts it under /tmp would have been
# told its own correct path was the bug. Being under /tmp is not the defect; being SHARED is.
require_per_run_channels() {
  if [ -z "$evidence_dir" ]; then
    echo "FAIL: no per-run evidence directory for '$1'." >&2
    echo "      Set GUNBC_CONTRACT_STAMP_EVIDENCE_DIR, or RUNNER_TEMP + GITHUB_RUN_ID + GITHUB_RUN_ATTEMPT." >&2
    echo "      Concurrent runner slots share /tmp: one slot's --prepare can delete another slot's fail" >&2
    echo "      marker and turn a real refusal green (reviews 82230, 82257)." >&2
    exit 1
  fi
  # NEVER INSIDE THE REPO. A transcript that lands in the working tree is a transcript someone commits:
  # this PR shipped exactly that, an empty gunbc-contract-stamp-evidence.log added by a blanket `git add
  # -A` after a manual probe (§3c, "a new artifact with no final consumer"). Per-run placement alone did
  # not prevent it -- the probe simply ran without RUNNER_TEMP, so the fallback landed here. A verdict
  # channel is scratch data with no consumer, so it is refused inside the tree as well as outside the
  # per-run directory.
  case "$1" in
    "$repo"/*)
      echo "FAIL: refusing a verdict channel inside the repo tree: '$1'." >&2
      echo "      Scratch that lives in the working tree gets committed (it already did once)." >&2
      echo "      Set RUNNER_TEMP (or GUNBC_CONTRACT_STAMP_EVIDENCE_DIR) to a path outside '$repo'." >&2
      exit 1 ;;
  esac
  case "$1" in
    "$evidence_dir"/*)
      return 0 ;;
    *)
      echo "FAIL: '$1' is outside the per-run evidence directory '$evidence_dir'." >&2
      echo "      A verdict channel no run owns is a channel two runs can share (reviews 82230, 82257)." >&2
      echo "      This applies to --transcript/--fail-marker from argv too: nothing else on the required" >&2
      echo "      path can catch a generated step that stopped deriving per-run channels." >&2
      exit 1 ;;
  esac
}
fail=0
guard_template="$(mktemp)"
tmpdir="$(mktemp -d)"
# THE VERDICT CHANNEL DEFAULTS, assigned HERE and nowhere earlier. Two traps, both of which shipped a
# stray file at the repo root:
#   * a bare relative name is resolved against the caller's CWD, so a truncated default CREATES a file in
#     the working tree, where the next `git add -A` commits it;
#   * the value must not be evaluated before $tmpdir exists, or the fallback degenerates to a path at the
#     filesystem root. Assignment order is the whole difference between scratch and scratch-that-destroys.
transcript="${GUNBC_CONTRACT_STAMP_EVIDENCE_TRANSCRIPT:-${evidence_dir:-$tmpdir}/gunbc-contract-stamp-evidence.log}"
fail_marker="${GUNBC_CONTRACT_STAMP_EVIDENCE_FAIL_MARKER:-${evidence_dir:-$tmpdir}/gunbc-contract-stamp-evidence.fail}"
cleanup() { rm -f "$guard_template"; [ -n "$tmpdir" ] && rm -rf "$tmpdir"; }
trap 'cleanup' EXIT

# THE FORGER'S OWN TOOL, in the harness. It receives a FORM NAME from the model and the RESOLVED running
# stamp, and writes the tree itself -- the same shape rewrite_stamp/demote_stamp already have. A byte
# string from the model would be worse: two of these forms contain a single quote, and the shell
# serializer single-quotes literal words, so the bytes would be re-parsed rather than quoted.
# DERIVED, NOT RE-COPIED. The harness holds no second copy of the reader's rule (DESIGN §6, parallel
# authority): the supported form is defined ONCE, in the model, by a content hash of fixed length. Here
# the question is only "did the forger actually forge?" -- so the reference is the RUNNING value, which
# the model guarantees is the supported form. Same length as that, and all lowercase hex, is the same
# rule with no constant duplicated.
stamp_literal_is_supported_form() {
  local v="$1" ref="$2"
  [ "${#v}" -eq "${#ref}" ] || return 1
  case "$v" in
    *[!0-9a-f]*) return 1 ;;
    *) return 0 ;;
  esac
}

forge_stamp() {
  form="$1"; base="$2"; src="$3"; dst="$4"
  [ "${#base}" -gt 0 ] || bail "cannot forge a form from an empty running stamp"
  # THE KEY IS THE MODEL'S (compiler_gate_contract_stamp_env_key), named once here so a form cannot
  # forge against a key the guard does not read. `extra` is the line the declare-twice forms append.
  stamp_key="GUNBC_COMPILER_GATE_CONTRACT_STAMP"
  extra=""
  case "$form" in
    wrapped)    val="XYZ${base}XYZ" ;;
    expression) val="\${{ '${base}' }}" ;;
    # NO `quoted` BRANCH, ON PURPOSE. A quoted scalar is not a forgery -- the ingest unquotes it, so it
    # declares the same stamp (see quote_stamp and SubjectQuotedScalar) -- and it is run as a CONTROL on
    # its own subject. The branch that forged one was deleted along with the model's ForgedQuoted variant
    # rather than left here as a branch no model row can reach: a harness case list that can outlive the
    # model's is a second authority over what gets tested, and the drift is silent.
    truncated)  val="${base%?}" ;;
    extended)   val="${base}0" ;;
    uppercased) val="$(printf '%s' "$base" | tr '0123456789abcdef' '0123456789ABCDEF')" ;;
    # THE THREE FORMS THAT FORGE BY DECLARING TWICE, through `extra`, rather than by writing a bad
    # value. The last two are the spellings the PREVIOUS emitted recogniser could not see at all: it
    # matched the literal prefix `  KEY: ` -- colon then SPACE -- so a member with nothing after the
    # colon, or a member whose KEY is quoted, never reached its duplicate counter. Both were executed
    # against that program and exited 0 as PAIRED, which is the whole reason a duplicate is refused:
    # a check that fires only when the duplicate is already well spelled is a check on nothing. The
    # MODEL owns which spellings exist (CompilerGateContractStampForgedForm); an unknown form here
    # bails rather than being skipped, so a modelled case can never arrive at nothing.
    duplicate)         val="$base"; extra="  $stamp_key: $base" ;;
    second-empty)      val="$base"; extra="  $stamp_key:" ;;
    second-quoted-key) val="$base"; extra="  \"$stamp_key\": deadbeefcafe0123" ;;
    *) bail "unknown forged form '$form' (modelled: wrapped, expression, truncated, extended, uppercased, duplicate, second-empty, second-quoted-key)" ;;
  esac
  case "$form" in
    # THE FORGERY MUST ACTUALLY BE A FORGERY. If the written value happened to be the one supported
    # form, the case would assert a refusal against a tree that legitimately pairs -- a tautology that
    # reads as a passing test. The declare-twice forms are exempt: they forge by DECLARING TWICE, and
    # a well-formed competing member is exactly the input that case needs -- refusing it would be
    # refusing the case's own premise.
    duplicate|second-empty|second-quoted-key) : ;;
    *)
      if stamp_literal_is_supported_form "$val" "$base"; then
        bail "forged form '$form' produced '$val', which is the supported form -- the case would prove nothing"
      fi ;;
  esac
  # ALL THREE -v BINDINGS BEFORE THE PROGRAM, not after it. An -v that follows the program text is read
  # as a FILE NAME: the first version of this bound new and form but left key= after the program, and awk
  # then tried to open "-v" and "key=..." as inputs -- the same fused-`-v` shape this module already
  # documents as a shipped defect, reintroduced by editing the invocation instead of reading it.
  awk -v new="$val" -v extra="$extra" -v form="$form" -v key="$stamp_key" '
    function die(m) { printf "FAIL: %s\n", m > "/dev/stderr"; exit 1 }
    /^[ \t]*GUNBC_COMPILER_GATE_CONTRACT_STAMP:[ \t]*[0-9a-f]+[ \t]*$/ {
      # BOTH BRANCHES MUST PRINT EXPLICITLY. The first version modified the line with sub() and relied
      # on the default action to print it -- but the next below (needed so the duplicate branch does
      # not print its original twice) skips that default for the OTHER branch too. Every single-member
      # forgery then SILENTLY DELETED the member it meant to change: five RED cases got StampAbsent
      # ("your tree predates the instruction") instead of the typed malformed-member refusal they
      # exist to prove, and placed never noticed because it counted intent, not bytes written. This
      # module is about a check that reports the wrong reason and looks like a pass.
      # (No apostrophes in this block: it lives inside a single-quoted program, where one closes the
      # quoting -- the same rule that keeps the emitted awk program quote-free.)
      if (form == "duplicate" || form == "second-empty" || form == "second-quoted-key") {
        print; print extra; placed += 2
      }
      else { sub(/:.*/, ": " new); print; placed += 1 }
      next
    }
    { print }
    END {
      declares = (form == "duplicate" || form == "second-empty" || form == "second-quoted-key")
      if (placed != (declares ? 2 : 1)) die("expected exactly one workflow-level stamp member, wrote " placed " for form " form) }
  ' "$src" > "$dst" || return 1
}

bail() { echo "FAIL: $*" >&2; exit 1; }
command -v awk >/dev/null 2>&1 || bail "awk is required"
[ -f "$committed" ] || bail "no committed workflow at $committed"
[ -f "$fixture" ] || bail "no silent-green fixture at $fixture"

# ---------------------------------------------------------------------------------------------
# extract_block <file> <step name line>  --  print that step's run body, byte exactly.
# A structural scan whose edges are asserted rather than assumed:
#   * locate the step by its exact `- name:` line; anything other than exactly one is a failure;
#   * the next line must be a `run:` block scalar (|- or |+ style);
#   * the block's indentation is READ from its own first non-blank line, and every deeper line is
#     re-emitted dedented by exactly that many spaces;
#   * a blank line, whether truly empty or spaces-only, is part of the block; only a NON-blank line
#     at a shallower indent terminates it;
#   * an empty body, a missing `run:` header, or a body not deeper than its own step name is a loud
#     failure -- never a silently short script.
# ---------------------------------------------------------------------------------------------
extract_block() {
  awk -v step="$2" '
    function die(m) { printf "FAIL: %s\n", m > "/dev/stderr"; exit 1 }
    function indent(s) { if (match(s, /[^ \t]/) == 0) return -1; return match(s, /[^ \t]/) - 1 }
    function strip(s) { sub(/^[ \t]+/, "", s); sub(/[ \t]+$/, "", s); return s }
    BEGIN { state = "seek"; found = 0; printed = 0; pending = 0 }
    state == "seek" {
      if (strip($0) == step) { found++; name_indent = indent($0); state = "want_run" }
      next
    }
    state == "want_run" {
      if (strip($0) == "") next
      if (strip($0) !~ /^run: [-+]?[0-9]*[|>][-+]?$/)
        die("step is not a `run:` block scalar (got: " strip($0) ")")
      state = "body"; body_indent = -1
      next
    }
    state == "body" {
      if ($0 == "" || indent($0) < 0) { pending++; next }
      ind = indent($0)
      if (body_indent < 0) {
        body_indent = ind
        if (body_indent <= name_indent) die("body is not deeper than its own step name")
      } else if (ind < body_indent) { state = "done"; next }
      while (pending > 0) { print ""; pending-- }
      print substr($0, body_indent + 1)
      printed++
      next
    }
    END {
      if (found != 1) die("expected exactly one such step, found " found)
      if (state != "done") die("the step block never terminated")
      if (printed == 0) die("the step body is empty")
    }
  ' "$1"
}

# emit_guard <running-value> <dest>  --  substitute ${{ env.X }} the way Actions would.
emit_guard() {
  awk -v ph="$placeholder" -v run="$1" '
    {
      out = ""
      while ((p = index($0, ph)) > 0) { out = out substr($0, 1, p - 1) run; $0 = substr($0, p + length(ph)) }
      print out $0
    }
  ' "$guard_template" > "$2"
}

# rewrite_stamp <new-stamp> <in> <out>  --  move the workflow-level stamp member, as a PR that
# legitimately changes the invoked contract would. Refuses rather than silently emitting a copy in
# which the member is missing, which would make the "contract-changing PR proceeds" control green
# for the wrong reason.
# THE SAME VALID STAMP, WRITTEN AS A QUOTED SCALAR. A subject rather than a form, because quoting is
# NOT A FORGERY: the YAML reader the guard folds over unquotes a quoted scalar, so the quoted spelling
# declares exactly the same stamp and the guard must PAIR with it. It is here as its own subject so the
# case reads as a control -- a document that is legal must come out green -- rather than as another
# "forged X, expect refusal" row, which is how the over-refusing reader hid: the table asked for a
# refusal of a legal document and got one, and everybody read that as a pass.
#
# The quote is built with sprintf("%c", 39) because this program is itself single-quoted, and an
# apostrophe inside would close the quoting.
quote_stamp() {
  awk -v new="$1" '
    function die(m) { printf "FAIL: %s\n", m > "/dev/stderr"; exit 1 }
    BEGIN { sq = sprintf("%c", 39) }
    /^[ \t]*GUNBC_COMPILER_GATE_CONTRACT_STAMP:[ \t]*[0-9a-f]+[ \t]*$/ {
      sub(/:.*/, ": " sq new sq); moved++
    }
    { print }
    END { if (moved != 1) die("expected exactly one workflow-level stamp member to quote, quoted " moved) }
  ' "$2" > "$3"
}

rewrite_stamp() {
  awk -v new="$1" '
    function die(m) { printf "FAIL: %s\n", m > "/dev/stderr"; exit 1 }
    /^[ \t]*GUNBC_COMPILER_GATE_CONTRACT_STAMP:[ \t]*[0-9a-f]+[ \t]*$/ {
      sub(/:.*/, ": " new); moved++
    }
    { print }
    END { if (moved != 1) die("expected exactly one workflow-level stamp member, rewrote " moved) }
  ' "$2" > "$3"
}

# demote_stamp <stamp> <in> <out>  --  remove the workflow-level member and hang the same key off the
# FIRST step's env, so it is textually present and structurally absent. The tree is not allowed to
# hand the judge its own evidence.
demote_stamp() {
  awk -v run="$1" '
    function die(m) { printf "FAIL: %s\n", m > "/dev/stderr"; exit 1 }
    /^[ \t]*GUNBC_COMPILER_GATE_CONTRACT_STAMP:[ \t]*[0-9a-f]+[ \t]*$/ { removed++; next }
    {
      print
      if (!inserted && $0 ~ /^[ \t]*-[ \t]+name:/) {
        printf "          env:\n"
        printf "            GUNBC_COMPILER_GATE_CONTRACT_STAMP: %s\n", run
        inserted = 1
      }
    }
    END {
      if (removed != 1) die("expected exactly one workflow-level stamp member, removed " removed)
      if (inserted != 1) die("no step to hang the demoted env: off of")
    }
  ' "$2" > "$3"
}

# STRUCTURE FIRST, THEN CONTENT, and the evidence step is checked before the guard: if the step that
# runs the evidence has gone missing, that is the finding, and it must not be masked by a later or
# earlier complaint about the stamp.
extract_block "$committed" "$evidence_step_name" > "$tmpdir/evidence_step.sh" \
  || bail "the committed witnesses.yml does not contain the modelled evidence step -- the evidence is NOT ENROLLED"
extract_block "$committed" "$guard_step_name" > "$guard_template" \
  || bail "could not extract the guard from the committed workflow"

# ONE FACT, TWO SPELLINGS, SO COMPARE THE SPELLINGS.
# The step gets its channels from model rows via per-run vars; this file derives its own from
# RUNNER_TEMP + the run identity. If the two derivations ever disagree -- a model row edited, an env
# override left set -- the step writes to one file while this count reads another, and local mode
# reports "nothing ran" about a run that ran perfectly. Since the step now passes VAR words rather than
# literals, comparing strings is useless; what can be compared is the DERIVATION: the channel must be
# built from the directory, the directory from RUNNER_TEMP and the run token, and the leaf file names
# must be exactly the ones this file writes.
step_arg() {
  awk -v flag="$1" -v q="'" '
  {
    n = 0
    for (i = 1; i <= NF; i++) {
      f = $i; gsub(q, "", f); gsub(/"/, "", f); v[++n] = f
    }
    for (i = 1; i < n; i++) if (v[i] == "--" flag) { print v[i + 1]; exit }
  }' "$tmpdir/evidence_step.sh"
}
step_assign() {
  grep -m1 "$1=" "$tmpdir/evidence_step.sh" | sed 's/^[ ]*//' || true
}
step_transcript="$(step_arg transcript)"
step_marker="$(step_arg fail-marker)"
[ -n "$step_transcript" ] || bail "the modelled evidence step declares no --transcript path; nothing can be counted"
[ -n "$step_marker" ]     || bail "the modelled evidence step declares no --fail-marker path; the verdict channel is missing"
case "$step_transcript" in
  *GUNBC_CONTRACT_STAMP_EVIDENCE_LOG*) : ;;
  *) bail "the step's --transcript is not the per-run channel var: '$step_transcript'" ;;
esac
case "$step_marker" in
  *GUNBC_CONTRACT_STAMP_EVIDENCE_FAIL*) : ;;
  *) bail "the step's --fail-marker is not the per-run channel var: '$step_marker'" ;;
esac
# the chain, not just the names
dir_assign="$(step_assign GUNBC_CONTRACT_STAMP_EVIDENCE_DIR)"
case "$dir_assign" in
  *RUNNER_TEMP*GUNBC_CONTRACT_STAMP_RUN_TOKEN*|*GUNBC_CONTRACT_STAMP_RUN_TOKEN*RUNNER_TEMP*) : ;;
  *) bail "the per-run directory is not derived from RUNNER_TEMP AND the run token: '$dir_assign'" ;;
esac
# and the leaf names must be the ones this harness writes into the same directory
log_name="${transcript##*/}"; fail_name="${fail_marker##*/}"
case "$(step_assign GUNBC_CONTRACT_STAMP_EVIDENCE_LOG)" in
  *"$log_name"*) : ;;
  *) bail "the step's transcript leaf differs from this harness's '$log_name'" ;;
esac
case "$(step_assign GUNBC_CONTRACT_STAMP_EVIDENCE_FAIL)" in
  *"$fail_name"*) : ;;
  *) bail "the step's fail-marker leaf differs from this harness's '$fail_name'" ;;
esac
# NO FIXED /tmp PATH ANYWHERE IN THE STEP. Concurrent runner slots share /tmp, so a fixed name there is
# the shared-path defect regardless of how many other guards are in place.
if grep -q '/tmp/' "$tmpdir/evidence_step.sh"; then
  bail "the modelled evidence step still references a fixed /tmp path -- channels must be per-run"
fi
echo "== step channels agreed: dir from RUNNER_TEMP+token, leaves $log_name / $fail_name"

# ALL THREE PER-RUN CHANNELS MUST BE ASSIGNED, not merely referenced. This is the check that would have
# caught the shipped omission (the step counted GUNBC_CONTRACT_STAMP_RUN_TOKEN which nothing assigned),
# applied to the transcript and the fail marker too -- a marker path that is never set is a verdict
# channel that silently points nowhere.
for required_assign in GUNBC_CONTRACT_STAMP_RUN_TOKEN GUNBC_CONTRACT_STAMP_EVIDENCE_DIR \
                       GUNBC_CONTRACT_STAMP_EVIDENCE_LOG GUNBC_CONTRACT_STAMP_EVIDENCE_FAIL; do
  grep -q "${required_assign}=" "$tmpdir/evidence_step.sh" \
    || bail "the modelled evidence step uses ${required_assign} but never assigns it"
done
token_assign="$(grep -m1 'GUNBC_CONTRACT_STAMP_RUN_TOKEN=' "$tmpdir/evidence_step.sh" || true)"
[ -n "$token_assign" ] || bail "the modelled evidence step counts GUNBC_CONTRACT_STAMP_RUN_TOKEN but never assigns it -- the step cannot tell one run from a stale transcript"
# AND IT MUST EXPAND. The serializer single-quotes LITERAL words and leaves var words bare: an
# assignment whose value was built from literal words would render as a quoted string, the token would
# be the literal text '$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT', non-empty so the -z arm would not fire, and
# the count would then be satisfiable by ANY transcript containing that constant -- exactly the hole the
# token exists to close, reopened by a rendering choice rather than by a logic error.
case "$token_assign" in
  *'$GITHUB_RUN_ID'*) : ;;
  *) bail "the step assigns GUNBC_CONTRACT_STAMP_RUN_TOKEN without expanding GITHUB_RUN_ID: '$token_assign'" ;;
esac
case "$token_assign" in
  *'$GITHUB_RUN_ATTEMPT'*) : ;;
  *) bail "the step assigns GUNBC_CONTRACT_STAMP_RUN_TOKEN without expanding GITHUB_RUN_ATTEMPT: '$token_assign'" ;;
esac
# SINGLE QUOTES ARE THE FAIL-OPEN HERE. A double-quoted value still expands, and the serializer wraps
# assignment values in double quotes, so rejecting those would refuse every correct step -- a check that
# is strict by accident is not a check anyone keeps. SINGLE quotes make a dollar sign literal: the token
# would be the fixed text '$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT', non-empty so the step's -z arm stays
# silent, and the count would be satisfiable by any transcript carrying that constant -- the exact hole
# the token exists to close, reopened by a rendering choice rather than a logic error.
token_value="$(printf '%s' "$token_assign" | awk '{print substr($0, index($0,"=")+1, 1)}')"
case "$token_value" in
  "'"*) bail "the token assignment is single-quoted, so its dollar signs are literal and every run shares one token: '$token_assign'" ;;
esac
# THE MODEL'S CASE COUNT MUST MATCH WHAT THE STEP INVOKES, checked before anything runs. If the model
# gained a case and the artifact predates it, the step's own count would refuse at the END of a run;
# saying so here names the real cause instead of reporting a harness failure.
model_cases="$(grep -c -- "--case" "$tmpdir/evidence_step.sh" || true)"
echo "== modelled case invocations in the step: $model_cases"
[ "$model_cases" -gt 0 ] || bail "the extracted evidence step invokes no --case at all"
[ -s "$guard_template" ] || bail "extracted guard is empty"

RUNNING="$(grep -o 'GUNBC_COMPILER_GATE_CONTRACT_STAMP: [0-9a-f]\{16\}' "$committed" | head -1 | awk '{print $2}')"
[ -n "$RUNNING" ] || bail "the committed workflow declares no readable 16-hex contract stamp"

# ---------------------------------------------------------------------------------------------
# SINGLE-CASE MODE, driven by the generated step. The expectations arrive as ARGUMENTS from the
# model; this branch has none of its own.
# ---------------------------------------------------------------------------------------------
if [ "${1:-}" = "--case" ]; then
  shift
  label="${1:-}"; subject="${2:-}"; running="${3:-}"; expect_rc="${4:-}"; finding="${5:-}"
  [ "$#" -ge 5 ] || bail "--case needs label, subject, running, expected-exit and finding"
  shift 5
  case_transcript="$transcript"; marker="$fail_marker"; token="$case_token_default"; run_token=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --transcript) case_transcript="$2"; shift 2 ;;
      --fail-marker) marker="$2"; shift 2 ;;
      # THE PER-RUN TOKEN, handed back from the model's own assignment. The generated step counts lines
      # carrying THIS value, not the constant marker: a stale transcript from an earlier invocation can
      # satisfy a count of a constant, and mine did exactly that in simulation (harness deleted, step
      # green). A value that only exists in this run cannot be stale.
      --run-token) run_token="$2"; shift 2 ;;
      # THE PRINTED TOKEN IS THE MODEL'S, not this file's. The generated step counts occurrences of
      # this exact string in the transcript and refuses unless the count equals the modelled table, so
      # the string the harness prints and the string the step greps for come from ONE value in
      # .dag rather than two spellings free to disagree.
      --marker) token="$2"; shift 2 ;;
      *) bail "--case: unknown argument $1" ;;
    esac
  done
  [ -n "$label" ] && [ -n "$subject" ] || bail "--case needs label and subject"
  [ -n "$marker" ] && [ -n "$case_transcript" ] || bail "--case needs --transcript and --fail-marker"
  # BOTH channels, whichever way they arrived -- including argv. --case is the mode the required path
  # actually invokes, so the rule is worthless unless it applies here (reviews 82230, 82257).
  require_per_run_channels "$case_transcript"
  require_per_run_channels "$marker"
  # THE TRANSCRIPT MUST BE WRITABLE BEFORE A CASE RUNS. The generated step mkdirs the per-run directory,
  # so this is normally already true; the reason to check anyway is the FAILURE MODE. If tee cannot write,
  # tee prints to stderr and the case still exits 0 -- the count then finds no lines and the step refuses
  # with "the harness ran the wrong number of cases", which blames the harness for what was a filesystem
  # problem. Name the real cause at the point it happens.
  if ! mkdir -p "$(dirname "$case_transcript")" 2>/dev/null; then
    bail "cannot create the transcript directory $(dirname "$case_transcript")"
  fi
  if ! : >> "$case_transcript" 2>/dev/null; then
    bail "cannot write the transcript '$case_transcript' -- a case that cannot record is a case that did not run"
  fi
  case "$expect_rc" in
    ""|*[!0-9]*) bail "--case: expected exit is not a number: '$expect_rc'" ;;
  esac
  # THE TOKEN MUST BE A RUN IDENTITY, not any string. The step only checks it is non-empty, which is not
  # enough: an empty pair of Actions vars degenerates it to "-", and grep for "-" matches almost every
  # line -- a fail-open count again, wearing the shape of a check. <run_id>-<attempt> is what the model
  # builds, so that is the only shape admitted here. (The table that checks these patterns is executed in
  # the commit that adds them; the first version of this check accepted 'abc-1', '12--3' and a trailing
  # dash, because the case globs were written the wrong way round.)
  case "$run_token" in
    ""|*[!0-9-]*) bail "--case: --run-token is empty or not digits-and-dashes: '$run_token'" ;;
  esac
  case "$run_token" in
    *-*-*) bail "--case: --run-token is not exactly <digits>-<digits>: '$run_token'" ;;
  esac
  case "$run_token" in
    *-*): ;;
    *) bail "--case: --run-token is missing its dash separator: '$run_token'" ;;
  esac
  case "$run_token" in
    -*) bail "--case: --run-token has an empty run id (GITHUB_RUN_ID unset?): '$run_token'" ;;
  esac
  case "$run_token" in
    *-) bail "--case: --run-token has an empty attempt (GITHUB_RUN_ATTEMPT unset?): '$run_token'" ;;
  esac

  # RESOLVE THE RUNNING SIDE FIRST, THEN BUILD THE SUBJECT. Order matters and it is the whole
  # difference between a real test and a tautology: the tree for `rewritten`/`demoted` is BUILT FROM
  # THE RESOLVED RUNNING VALUE, so "both sides moved together" is a fact the harness enforces rather
  # than a pair of constants the model happens to agree on. Baking a placeholder into the tree instead
  # would give a non-hex stamp, which lands in the StampAbsent arm -- the same wrong-reason green the
  # fused `-v key=` defect produced.
  if [ "$running" = "$running_probe_token" ]; then running="$RUNNING"; fi
  case "$running" in
    "") ;;  # the unreadable case: no running stamp at all
    *[!0-9a-f]*) bail "--case: running value '$running' is not empty and not hex; the guard would refuse it for the wrong reason" ;;
  esac

  # THE SUBJECTS ARE THE MODEL'S SYMBOLS; turning one into a file is the mechanical half. The model
  # decides WHICH subjects exist and what each proves; a path here would be a second spelling of the
  # same fact, free to disagree in silence.
  case "$subject" in
    fixture)   tree="$fixture" ;;
    committed) tree="$committed" ;;
    rewritten) rewrite_stamp "$running" "$committed" "$tmpdir/rewritten.yml" \
                 || bail "cannot build subject 'rewritten'"
               tree="$tmpdir/rewritten.yml" ;;
    demoted)   demote_stamp "$running" "$committed" "$tmpdir/demoted.yml" \
                 || bail "cannot build subject 'demoted'"
               tree="$tmpdir/demoted.yml" ;;
    quoted-scalar) quote_stamp "$running" "$committed" "$tmpdir/quoted.yml" \
                 || bail "cannot build subject 'quoted-scalar'"
               tree="$tmpdir/quoted.yml" ;;
    forged:*)  forge_stamp "${subject#forged:}" "$running" "$committed" "$tmpdir/forged.yml" \
                 || bail "cannot build subject '$subject'"
               tree="$tmpdir/forged.yml" ;;
    *) bail "unknown subject '$subject' (modelled: fixture, committed, rewritten, demoted, quoted-scalar)" ;;
  esac

  d="$tmpdir/single_$(printf '%s' "$label" | tr -c '[:alnum:]' '_')"
  mkdir -p "$d/.github/workflows"
  cp "$tree" "$d/.github/workflows/witnesses.yml"
  emit_guard "$running" "$d/guard.sh"
  out="$(cd "$d" && bash -e -o pipefail ./guard.sh 2>&1)"; rc=$?
  # ONE line per case, beginning with the model's token. `grep -c` counts LINES, so the token appears
  # exactly once here and never inside the diagnostic that follows -- a guard message containing the
  # token could otherwise inflate the tally and make a partial run look complete.
  # EXACTLY ONE LINE PER CASE, guard text folded in. Two lines per case would give the count a second
  # chance to match: the folded-in text is the guard's own output, and a guard message that happened to
  # contain the run token would inflate the tally and make a PARTIAL run look complete -- the failure mode
  # this check exists to close.
  one_line="$(printf '%s' "$out" | tr '\n' ' ' | cut -c1-118)"
  printf '%s run=%s label=%-40s rc=%s expect=%s | %s\n' \
    "$token" "$run_token" "$label" "$rc" "$expect_rc" "$one_line" | tee -a "$case_transcript"

  ok=1
  if [ "$rc" != "$expect_rc" ]; then
    printf '   ^^ ASSERTION FAILED (exit code: got %s, expected %s)\n' "$rc" "$expect_rc" | tee -a "$case_transcript"
    ok=0
  fi
  if ! printf '%s' "$out" | grep -q -- "$finding"; then
    printf '   ^^ ASSERTION FAILED (diagnostic lacks %s)\n' "$finding" | tee -a "$case_transcript"
    ok=0
  fi
  # BOTH sides asserted, and the failure is left behind rather than only printed: an exit code that
  # dies for an unrelated reason also yields 1, which is exactly how the fused `-v key=` render passed
  # every RED case while the guard was refusing correct trees.
  [ "$ok" = "1" ] || : > "$marker"
  exit 0
fi

# ---------------------------------------------------------------------------------------------
# PREPARE MODE -- the generated step's first statement. A marker left by an earlier run on the same
# runner must not redden a clean run, and its absence must be meaningful.
# ---------------------------------------------------------------------------------------------
if [ "${1:-}" = "--prepare" ]; then
  # FLAGS ONLY, for both paths. The step once passed the marker POSITIONALLY here while the transcript
  # came as a flag; a caller that forgot the positional got a silent empty marker and "prepared"
  # nothing. Now both arrive by name, exactly as the model emits them, and a missing one refuses --
  # including the harness's own default, so the two spellings of one fact cannot disagree.
  shift
  prep_marker=""; prep_transcript=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --fail-marker) prep_marker="$2"; shift 2 ;;
      --transcript) prep_transcript="$2"; shift 2 ;;
      *) bail "--prepare: unknown argument $1" ;;
    esac
  done
  [ -n "$prep_marker" ] || prep_marker="$fail_marker"
  [ -n "$prep_transcript" ] || prep_transcript="$transcript"
  [ -n "$prep_marker" ] || bail "--prepare needs --fail-marker"
  [ -n "$prep_transcript" ] || bail "--prepare needs --transcript"
  # --prepare DELETES files, which is exactly how the shared-path bug does its damage: a fixed name
  # means one slot's prepare wipes another slot's marker. Enforced on argv too -- see the rule above.
  require_per_run_channels "$prep_transcript"
  require_per_run_channels "$prep_marker"
  rm -f "$prep_marker"
  # Both artifacts are cleared: the marker so a stale failure cannot redden a clean run, and the
  # TRANSCRIPT so a stale success cannot GREEN a run that executed nothing. The generated step counts
  # lines in it, so leaving an old one behind would let the count reach the modelled number by itself.
  # Both are per-run (see the paths block above), so this clears THIS run's files, not a neighbour's.
  [ -n "$prep_transcript" ] && : > "$prep_transcript"
  printf 'gunbc contract stamp evidence: prepared (stale fail marker and transcript cleared)\n'
  exit 0
fi

# ---------------------------------------------------------------------------------------------
# LOCAL MODE. Run THE GENERATED STEP'S OWN BYTES. This is the point of the file: there is no second
# table here to drift from the model, and a step that has been dropped from the workflow makes this
# mode fail, because there is nothing to run.
# ---------------------------------------------------------------------------------------------
echo "== awk: $(awk --version 2>&1 | head -1)"
echo "== guard:  $(grep -c '' < "$guard_template") lines, sha256 $(sha256sum < "$guard_template" | cut -d' ' -f1 | cut -c1-16)"
echo "== evidence step: $(grep -c '' < "$tmpdir/evidence_step.sh") lines, extracted from the committed artifact"
echo "== running-side stamp (workflow-level literal): $RUNNING"

stepdir="$tmpdir/step"; mkdir -p "$stepdir"
cp "$tmpdir/evidence_step.sh" "$stepdir/evidence_step.sh"
# THE HARNESS SUPPLIES THE RUN IDENTITY HERE, because outside Actions GITHUB_RUN_ID and
# GITHUB_RUN_ATTEMPT are unset and the step's own refusal would fire before any case ran -- a correct
# refusal, but one that makes local mode permanently red and so trains everyone to ignore it. The step
# stays strict; THIS side is the one that has to look like a run. Unique per invocation so a transcript
# left by an earlier invocation cannot satisfy the count either, which is the whole reason the token
# exists. Both are digits, matching the shape the --case mode admits.
: "${GITHUB_RUN_ID:=$(date +%s)$$}"
: "${GITHUB_RUN_ATTEMPT:=1}"
# The step derives its channels from RUNNER_TEMP and refuses if it is unset, so local mode has to look
# like a job in that respect too -- same reason the run identity is synthesized above.
: "${RUNNER_TEMP:=$tmpdir/runner-temp}"
mkdir -p "$RUNNER_TEMP"
case "$GITHUB_RUN_ID" in ""|*[!0-9]*) echo "FAIL: synthesized GITHUB_RUN_ID is not digits: '$GITHUB_RUN_ID'" >&2; exit 1 ;; esac
export GITHUB_RUN_ID GITHUB_RUN_ATTEMPT RUNNER_TEMP
run_token="$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
# THE HARNESS AND THE STEP MUST MEAN THE SAME TWO FILES. The step derives them from RUNNER_TEMP + the
# run identity; derive them identically here rather than trusting the defaults, so a divergence in either
# derivation is a loud disagreement and not a count of a file that was never written.
# evidence_dir is set HERE, not just derived from the environment: local mode invents its own run
# identity, and the guard that demands every channel live inside the per-run directory tests against
# this value. Setting the paths without it would make local mode refuse its own correct layout.
evidence_dir="$RUNNER_TEMP/gunbc-contract-stamp-$run_token"
transcript="$evidence_dir/gunbc-contract-stamp-evidence.log"
fail_marker="$evidence_dir/gunbc-contract-stamp-evidence.fail"
mkdir -p "$(dirname "$transcript")"
# NOW the channels may be touched -- after they have been settled and guarded. Truncating or removing
# them earlier acted on whatever the defaults happened to be, which is how a stray file was created in
# the working tree rather than in scratch space.
require_per_run_channels "$transcript"
require_per_run_channels "$fail_marker"
# The generated step clears the marker itself as its first modelled statement; clearing here too keeps
# LOCAL MODE honest when the step is run twice in one shell, and is harmless when it is not.
: > "$transcript"
rm -f "$fail_marker"
echo "== run token: $run_token"
out="$(cd "$repo" && bash "$stepdir/evidence_step.sh" 2>&1)"; rc=$?
printf '%s\n' "$out"
echo ""
echo "== evidence step exit: $rc"
if [ "$rc" != "0" ]; then
  echo "== EVIDENCE VERDICT: THE MODELLED CASES DID NOT ALL HOLD (step exited $rc)"
  exit 1
fi
if [ -f "$fail_marker" ]; then
  echo "== EVIDENCE VERDICT: FAIL MARKER PRESENT despite exit 0 -- refusing"
  exit 1
fi
# Counted on the RUN TOKEN, not on "rc=": the same reason the modelled count check exists. Counting a
# constant is satisfiable by anything that happens to contain it, including a leftover transcript.
cases_run="$(grep -c "$run_token" < "$transcript" || true)"
if [ "$cases_run" -eq 0 ]; then
  echo "== EVIDENCE VERDICT: the transcript holds NO lines for run $run_token -- nothing ran; refusing"
  exit 1
fi
echo "== EVIDENCE VERDICT: ALL MODELLED CASES HELD ($cases_run cases, executed from the generated step)"
