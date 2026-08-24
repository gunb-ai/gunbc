#!/usr/bin/env bash
# SCAFFOLD -- hand-authored shell orchestration, PROBE-ONLY, awaiting an operator verdict.
# dissolve-on: gunbc bash-emit #5828, or a modeled cssl_probe transport in .dag, drives this sweep
#              from the substrate; at that point this file is DELETED, not ported.
#
# THE HONEST CLASSIFICATION, corrected after review 55258 (codex/gpt-5.6-sol) on gunbc#9064. An
# earlier revision of this header claimed composing an existing scaffold "adds no new debt beyond
# the one it composes". That was self-authorized dissolution -- DESIGN.md names it by that name --
# and it was wrong on its own terms: a second removable unit is a second obligation whoever deletes
# the first must also find, and a trigger the author writes is a lifecycle fact, never permission.
# This IS a new scaffold. It is DESIGN §6's out-of-band-actuation tell (raw shell implementing
# semantics the substrate should express) and it lands only if the operator approves that exact
# exception; absent that verdict the correct disposition is deletion, and the boards this file
# already produced survive it intact -- they are committed data with committed sha256s, and
# reproducing them needs curated_cargo_probe_one.sh, which is in tree, plus the roster read this
# file mechanises.
#
# WHAT WOULD BE LOST BY DELETING IT, stated so the operator is deciding against something real
# rather than against a convenience: the roster read below is the only mechanised defence against
# the denominator failure described further down, and without it every future cohort re-derives
# its own module list by hand -- which is exactly the enumeration that produced three wrong
# populations in this fleet in one evening.
#
# WHAT THIS IS: one board per frontier module, taken in ONE process at ONE ref, with the module
# list and each module's shim_lib_rel READ FROM THE ROSTER AUTHORITY
# (dag/tools/self_host_module_behavioral_transport_roster.dag) rather than from a filename glob,
# a grep on a field name, or a hand-copied list. Each of those three convenient enumerations was
# tried in this fleet in one evening and each produced a population close enough to pass a glance
# and wrong in the denominator: 17 where the roster says 16, a glob reported as roster membership,
# and a zero where the true answer was five. The authoritative enumeration is the inconvenient
# one, so it is mechanised here rather than left to the caller's diligence.
#
# shim_lib_rel IS NOT DEFAULTABLE. 12 of the 16 rows carry the empty string and 4 do not, so
# guessing empty is right 12 times and silently wrong 4 -- which is worse than being uniformly
# wrong, because the guess earns trust before it betrays it. A wrong-lane lib.rs REPLACES the
# assembled lib.rs entirely and can cargo-green a crate that never compiled (the PHANTOM-GREEN
# hazard curated_cargo_probe_one.sh documents); an absent required shim produces out-of-scope
# refusals that read as module findings.
#
# USAGE:  PROBE_KEEP_LOG_DIR=<dir> [SWEEP_BASE_SHA=<sha>] curated_cargo_board_cohort.sh [module ...]
#         With no arguments, boards every roster row. With arguments, boards exactly the named
#         module paths, refusing any that the roster does not declare.
# OUTPUT: one TSV row per module on stdout (curated_cargo_probe_one.sh's 13-column row) and one
#         <module>.cargo.log per module in PROBE_KEEP_LOG_DIR.
#
# FAIL-CLOSED: a module whose probe line-stops (EMIT_REFUSE / HARNESS_REFUSE) still prints its
# row, and the cohort continues so the remaining boards are taken, but the exit code is non-zero
# and the refused modules are named on stderr. A cohort that quietly boards 12 of 15 and reports
# a weighting over "the frontier" is the empty-observation narrow DESIGN.md names -- the missing
# boards must be visible in the receipt, not inferable from a short table.
set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"
ROSTER="$ROOT/dag/tools/self_host_module_behavioral_transport_roster.dag"

if [[ -z "${PROBE_KEEP_LOG_DIR:-}" ]]; then
  echo "curated_cargo_board_cohort: REFUSED — PROBE_KEEP_LOG_DIR is required; a board whose log is discarded cannot be re-partitioned" >&2
  exit 2
fi
if [[ ! -f "$ROSTER" ]]; then
  echo "curated_cargo_board_cohort: REFUSED — roster authority not found at $ROSTER" >&2
  exit 2
fi

# The authority read. `module_path:` and `shim_lib_rel:` are the first two fields of every
# ModuleBehavioralTransportConfig row and appear in that order; the type declaration's own
# `module_path: String` line carries no quoted literal and so cannot be mistaken for a row.
ROSTER_PAIRS="$(awk '
  /module_path: *"/ { match($0, /"[^"]*"/); mod = substr($0, RSTART+1, RLENGTH-2); next }
  /shim_lib_rel: *"/ { if (mod != "") { match($0, /"[^"]*"/); printf "%s\t%s\n", mod, substr($0, RSTART+1, RLENGTH-2); mod = "" } }
' "$ROSTER")"

if [[ -z "$ROSTER_PAIRS" ]]; then
  echo "curated_cargo_board_cohort: REFUSED — roster parse produced zero rows; the authority's shape changed" >&2
  exit 2
fi

declare -a COHORT=()
if [[ $# -eq 0 ]]; then
  while IFS= read -r line; do COHORT+=("$line"); done <<< "$ROSTER_PAIRS"
else
  for want in "$@"; do
    row="$(grep -m1 -P "^\Q$want\E\t" <<< "$ROSTER_PAIRS" || true)"
    if [[ -z "$row" ]]; then
      echo "curated_cargo_board_cohort: REFUSED — $want is not a roster row; boarding a non-member contaminates the cohort denominator" >&2
      exit 2
    fi
    COHORT+=("$row")
  done
fi

echo "curated_cargo_board_cohort: roster declares $(wc -l <<< "$ROSTER_PAIRS") modules; boarding ${#COHORT[@]}" >&2

REFUSED=()
for row in "${COHORT[@]}"; do
  module="${row%%$'\t'*}"
  shim="${row#*$'\t'}"
  echo "curated_cargo_board_cohort: BOARD_BEGIN $module shim='${shim}'" >&2
  if ! CSSL_STD_SEED_LINK=1 PROBE_KEEP_LOG_DIR="$PROBE_KEEP_LOG_DIR" \
       bash "$SCRIPT_DIR/curated_cargo_probe_one.sh" "$module" "$shim"; then
    REFUSED+=("$module")
  fi
  echo "curated_cargo_board_cohort: BOARD_END $module" >&2
done

if [[ ${#REFUSED[@]} -gt 0 ]]; then
  echo "curated_cargo_board_cohort: ${#REFUSED[@]} of ${#COHORT[@]} boards line-stopped: ${REFUSED[*]}" >&2
  exit 1
fi
