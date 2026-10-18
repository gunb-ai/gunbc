#!/usr/bin/env bash
# Execute the contract-stamp guard against the real bytes, and fail if it does not refuse what it
# is supposed to refuse.
# SCAFFOLD - dissolve-on: the restoration_trigger of gunbc.rung_drop v1_required_lanes_withdrawn --
# "on every landing revision, a binary built from an emission of v2 ... judges this population on the
# required path with a discriminating red". That is THE trigger, cited by symbol so this carrier and the
# drop retire together rather than separately (manager ruling on review 82166, 2026-10-18). When that
# drop is retired, the modelled rows in compiler_gate_contract_stamp_evidence_cases become a `gunbc test`
# row in gunbc.instrument_targets and run the guard through shell.Exec.RunArgvOutcome, and THIS FILE IS
# DELETED. A runner-side pre-build reader of a committed workflow member (GitHub's own workflow_sha)
# would retire the guard's read too, but it is not what dissolves the SPAWNING residual here.
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
#   --prepare <path> Clear a stale fail marker left by an earlier run on the same runner.
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
transcript="${GUNBC_CONTRACT_STAMP_EVIDENCE_TRANSCRIPT:-/tmp/gunbc-contract-stamp-evidence.log}"
fail_marker="${GUNBC_CONTRACT_STAMP_EVIDENCE_FAIL_MARKER:-/tmp/gunbc-contract-stamp-evidence.fail}"

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

# DERIVE THE STEP'S OWN PATHS, then demand they agree with this harness's defaults. The step gets its
# transcript and marker paths from model rows (compiler_gate_contract_stamp_evidence_transcript_path /
# _fail_marker_path); this file has its own hardcoded defaults. If those two spellings ever disagree --
# a model row edited, an env override left set -- the step writes to one file while this count reads
# another, and local mode reports "nothing ran" for a run that ran perfectly. One fact, two spellings
# (§3): counted, not assumed. Bail names the override rather than leaving a mystery.
# Both quoting shapes, because the serializer quotes a literal word but NOT a var word: the same flag
# appears as '--transcript /path' and as "--transcript "$VAR"". Matching only the quoted shape would
# report "declares no --transcript path" for a perfectly good step -- a false refusal, which is the
# worse direction for a check that is supposed to be honest rather than strict-by-accident.
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
step_transcript="$(step_arg transcript)"
step_marker="$(step_arg fail-marker)"
[ -n "$step_transcript" ] || bail "the modelled evidence step declares no --transcript path; nothing can be counted"
# THE STEP MUST ASSIGN WHAT IT COUNTS. Found by execution, not by the claim that should have caught it:
# the model once built the token assignment and never appended it to the statement list, so the step
# grepped for a variable nothing had set and its own fail-closed -z arm refused every run. A claim would
# have caught it too -- but no claim is evaluated on the required path, so the check lives HERE, in the
# thing that actually runs.
if ! grep -q 'GUNBC_CONTRACT_STAMP_RUN_TOKEN=' "$tmpdir/evidence_step.sh"; then
  bail "the modelled evidence step counts GUNBC_CONTRACT_STAMP_RUN_TOKEN but never assigns it -- the step cannot tell one run from a stale transcript"
fi
[ -n "$step_marker" ]     || bail "the modelled evidence step declares no --fail-marker path; the verdict channel is missing"
if [ "$step_transcript" != "$transcript" ]; then
  echo "FAIL: the step writes its transcript to '$step_transcript' but this harness counts '$transcript'." >&2
  echo "      Two spellings of one fact; run with GUNBC_CONTRACT_STAMP_EVIDENCE_TRANSCRIPT='$step_transcript'" >&2
  exit 1
fi
if [ "$step_marker" != "$fail_marker" ]; then
  echo "FAIL: the step sets its fail marker at '$step_marker' but this harness inspects '$fail_marker'." >&2
  echo "      Two spellings of one fact; run with GUNBC_CONTRACT_STAMP_EVIDENCE_FAIL_MARKER='$step_marker'" >&2
  exit 1
fi
echo "== step paths agreed: transcript=$step_transcript marker=$step_marker"
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
  marker="${2:-}"
  [ -n "$marker" ] || bail "--prepare needs the fail-marker path"
  shift 2
  prep_transcript=""
  while [ "$#" -gt 0 ]; do
    case "$1" in
      --transcript) prep_transcript="$2"; shift 2 ;;
      *) bail "--prepare: unknown argument $1" ;;
    esac
  done
  rm -f "$marker"
  # Both /tmp artifacts are cleared: the marker so a stale failure cannot redden a clean run, and the
  # TRANSCRIPT so a stale success cannot GREEN a run that executed nothing. The generated step counts
  # lines in it, so leaving an old one behind would let the count reach the modelled number by itself.
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
case "$GITHUB_RUN_ID" in ""|*[!0-9]*) echo "FAIL: synthesized GITHUB_RUN_ID is not digits: '$GITHUB_RUN_ID'" >&2; exit 1 ;; esac
export GITHUB_RUN_ID GITHUB_RUN_ATTEMPT
run_token="$GITHUB_RUN_ID-$GITHUB_RUN_ATTEMPT"
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
