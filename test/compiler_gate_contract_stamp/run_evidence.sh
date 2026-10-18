#!/usr/bin/env bash
# Execute the contract-stamp guard against the real bytes, and fail if it does not refuse what it
# is supposed to refuse.
# SCAFFOLD - dissolve-on: TWO capabilities, named separately because they retire DIFFERENT halves of
# this file. Naming only one is what review 82230 filed, and review 82166 filed its mirror image, so the
# pair is on the record rather than one of them being traded away.
#   (1) orch-emitted for the RESIDUAL SHELL: the orchestration-to-shell bash-emit capability, i.e. node
#       kinds in v2.extdeps.languages.bash_build (rendered by v2.workflow.bash_emit / bash_emit_stmts)
#       for WHAT THIS FILE DOES -- extract a committed workflow member, scan it structurally, drive
#       argv, invoke awk. Everything this PR could express that way already IS expressed that way: the
#       guard step and the evidence step are node trees, and that is why they carry dissolve-on lines in
#       the emitted YAML. What has no node kind is the extraction and the awk program, and that is the
#       residual sitting in this file. When those node kinds land, this file becomes an emitted step.
#   (2) gunbc.rung_drop v1_required_lanes_withdrawn for the TEST PROCESS: its restoration_trigger --
#       "on every landing revision, a binary built from an emission of v2 ... judges this population on
#       the required path with a discriminating red" -- is the condition under which the modelled rows in
#       compiler_gate_contract_stamp_evidence_cases become a `gunbc test` row in gunbc.instrument_targets
#       and run the guard through shell.Exec.RunArgvOutcome. When THAT lands, this file is DELETED rather
#       than emitted, because nothing needed executing by hand any more.
# Cited by symbol so carrier and drop retire together rather than separately (manager ruling on review
# 82166, 2026-10-18). A runner-side pre-build reader of a committed workflow member (GitHub's own
# workflow_sha) would retire the guard's READ as well; it is not what dissolves the SPAWNING residual.
#
# HONEST STATE OF THE RESIDUAL, corrected after review 82166 (claude/opus[1m]) caught me claiming a
# capability gap that does not exist. I wrote that "the substrate has no process-exec surface at all"
# and cited Command::new/ProcessBuilder. THAT WAS FALSE: `git grep -l shell.Exec.Run dag src` returns
# dag/extdeps/shell/exec.dag, which declares shell.Exec.RunArgv / RunArgvOutcome / RunArgvStdin with
# ProcessObservation {exit_code, stdout, stderr} -- exactly the spawn-and-observe I said was missing --
# and dag/test/claim/machine_intake/fan_observe_witness.dag:297 CALLS it from a `test fn`. My grep used
# the wrong names, and the scaffold's dissolution trigger therefore named a capability that has already
# landed, which is not an honest trigger (§5: "a dissolution condition describes how admitted debt ends;
# it does not authorize creating it").
#
# WHAT I DID ABOUT IT, and why the file is still here. Everything that does NOT need a process has been
# MOVED into .dag, per §5: which cases exist, which subject each reads, the expected exit code and the
# expected finding for each are rows (compiler_gate_contract_stamp_evidence_cases in
# gunbc.witness.compiler_gate_contract_stamp), emitted into the generated step as one command node per
# row and passed here as argv -- so this file holds NO table that could drift from the model, and no
# modelling that the .dag route already covers. What remains here is the ACT OF SPAWNING inside a step
# that runs BEFORE the subject binary is built. The modelled RunArgv route is a claim, and a claim is
# not executed anywhere on the required pull_request path today: `heal.yml` runs `claim_executor
# --regen-round-cost` (an instrument, not a gate), and the instrument that executes `gunbc test <label>`
# rows -- instrument-dispatch.yml:9 -- is `workflow_dispatch`-only. Moving the evidence into claims
# would therefore move it OFF the required path, which is the exact defect review 82145 filed as
# blocking ("the RED and its controls were ATTESTED, not enrolled"; §4b(4)). So the residual is
# retained as an admitted, marked scaffold whose trigger is now an honest capability, and the enrolment
# stays where it is observable: in the generated witnesses.yml step, on the required path.

# WHO OWNS WHAT. The case TABLE -- which cases exist, which subject each reads, what exit code and
# which diagnostic each must produce -- is modelled in .dag, at
# dag/gunbc/witness/compiler_gate_contract_stamp.dag (compiler_gate_contract_stamp_evidence_cases),
# and it is EMITTED into .github/workflows/witnesses.yml as one command node per row. This script
# contains no table of expectations, because a second copy of a table is a second authority: review
# 82145 (claude/opus[1m], REQUEST_CHANGES) filed exactly that as blocking, together with "executed by
# nothing", and DESIGN section 5 prefers construction over validation -- the emitted bytes should be
# REQUIRED to agree with the model, not checked against it afterwards.
#
# What remains here is the ACT OF SPAWNING inside a step that runs before the subject binary exists --
# stated once, above, rather than twice: review 82188 (claude/opus[1m], REQUEST_CHANGES) filed as
# blocking that this header contradicted itself, because this paragraph still asserted the retracted
# "no Command::new / ProcessBuilder surface anywhere" claim below the correction that says otherwise.
# A scaffold whose stated reason contradicts itself has no honest dissolution condition (§5), so the
# duplicate is deleted rather than annotated.
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
# THE PATHS ARE PER-RUN, AND DERIVED HERE EXACTLY AS THE MODEL DERIVES THEM (review 82230).
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
evidence_dir="${GUNBC_CONTRACT_STAMP_EVIDENCE_DIR:-}"
if [ -z "$evidence_dir" ]; then
  if [ -n "${RUNNER_TEMP:-}" ] && [ -n "${GITHUB_RUN_ID:-}" ] && [ -n "${GITHUB_RUN_ATTEMPT:-}" ]; then
    evidence_dir="$RUNNER_TEMP/gunbc-contract-stamp-$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
  fi
fi
transcript="${GUNBC_CONTRACT_STAMP_EVIDENCE_TRANSCRIPT:-${evidence_dir:+$evidence_dir/}gunbc-contract-stamp-evidence.log}"
fail_marker="${GUNBC_CONTRACT_STAMP_EVIDENCE_FAIL_MARKER:-${evidence_dir:+$evidence_dir/}gunbc-contract-stamp-evidence.fail}"
# WHERE THE PER-RUN REQUIREMENT BITES, and where it does not. It is checked in two places:
#   * the STEP's bytes -- the step must derive its channels from the per-run vars and must contain no
#     /tmp/ path at all (the derivation checks below);
#   * this file's OWN defaults -- see require_per_run_channels, called once a mode has settled its paths.
# It is deliberately NOT imposed on paths handed to us as argv. A caller who names a scratch file for one
# --case invocation is doing something legitimate (and my own regeneration dispatch does exactly that);
# the thing that must never share a path with a neighbour is a verdict nobody chose. Enforcing it on argv
# too would refuse the model's own well-formed invocation only after the model had been proven correct
# elsewhere, which is strictness with no failure mode behind it.
require_per_run_channels() {
  # Called with the paths a mode settled on when those paths came from THIS file's defaults rather than
  # from argv. A fixed name here is the review 82230 bug: concurrent runner slots share /tmp, so one
  # slot's --prepare deletes another slot's fail marker and a real refusal goes green.
  #
  # THE TEST IS MEMBERSHIP OF THE PER-RUN DIRECTORY, not "is not under /tmp". The first version rejected
  # any path containing /tmp/, which would have refused the very layout this fix exists to require --
  # RUNNER_TEMP is a directory someone else chooses, and a runner (or a developer) that puts it under
  # /tmp would have been told its own correct path was the bug. Being under /tmp is not the defect;
  # being SHARED is.
  if [ -z "$evidence_dir" ]; then
    echo "FAIL: no per-run evidence directory for '$1'." >&2
    echo "      Set GUNBC_CONTRACT_STAMP_EVIDENCE_DIR, or RUNNER_TEMP + GITHUB_RUN_ID + GITHUB_RUN_ATTEMPT." >&2
    echo "      Concurrent runner slots share /tmp: one slot's --prepare can delete another slot's fail" >&2
    echo "      marker and turn a real refusal green (review 82230)." >&2
    exit 1
  fi
  case "$1" in
    "$evidence_dir"/*)
      return 0 ;;
    *)
      echo "FAIL: '$1' is outside the per-run evidence directory '$evidence_dir'." >&2
      echo "      A verdict channel no run owns is a channel two runs can share (review 82230)." >&2
      exit 1 ;;
  esac
}
fail=0
guard_template="$(mktemp)"
tmpdir="$(mktemp -d)"
cleanup() { rm -f "$guard_template"; [ -n "$tmpdir" ] && rm -rf "$tmpdir"; }
trap 'cleanup' EXIT

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

# ONE FACT, TWO SPELLINGS, SO COMPARE THE SPELLINGS (review 82230 follow-up).
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
# the review 82230 defect regardless of how many other guards are in place.
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
  from_argv=0
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --transcript) case_transcript="$2"; from_argv=1; shift 2 ;;
      --fail-marker) marker="$2"; from_argv=1; shift 2 ;;
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
  # Paths the CALLER chose are the caller's business (a scratch file for one invocation is legitimate).
  # Paths this file invented for itself must be per-run, or two slots share one verdict channel.
  [ "$from_argv" = "1" ] || require_per_run_channels "$case_transcript"
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
    *) bail "unknown subject '$subject' (modelled: fixture, committed, rewritten, demoted)" ;;
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
  prep_from_argv=1
  [ -n "$prep_marker" ] || { prep_marker="$fail_marker"; prep_from_argv=0; }
  [ -n "$prep_transcript" ] || { prep_transcript="$transcript"; prep_from_argv=0; }
  [ -n "$prep_marker" ] || bail "--prepare needs --fail-marker"
  [ -n "$prep_transcript" ] || bail "--prepare needs --transcript"
  # --prepare DELETES files, so it is where the shared-path bug does real damage: a fixed name means
  # one slot's prepare wipes another slot's marker. Defaults must be per-run; argv stays the caller's.
  [ "$prep_from_argv" = "1" ] || require_per_run_channels "$prep_transcript"
  rm -f "$marker"
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
: > "$transcript"
# The generated step clears the marker itself as its first modelled statement; clearing here too keeps
# LOCAL MODE honest when the step is run twice in one shell, and is harmless when it is not.
rm -f "$fail_marker"
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
transcript="$RUNNER_TEMP/gunbc-contract-stamp-$run_token/gunbc-contract-stamp-evidence.log"
fail_marker="$RUNNER_TEMP/gunbc-contract-stamp-$run_token/gunbc-contract-stamp-evidence.fail"
mkdir -p "$(dirname "$transcript")"
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
