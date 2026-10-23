#!/bin/bash
# BISECT, the parent's ruling (msg_0e84a10b: "bisect first"). The required-regen route refuses with
# 6 hard diagnostics of a shape I have never seen -- "a 'Present' arm decides from the scrutinee's
# type ... the scrutinee's type is unresolved, typically because a generic variant upstream was
# considered". Three arms separate whose code it is. Each prints EVERY diagnostic verbatim, uncut,
# and each rebuilds claim_executor because the binary's behaviour is the installed seed.
#   A  clean main:        main's 04_infer.dag + main's mirrors          <- if this refuses, the
#                                                                        route is broken independent
#                                                                        of this PR
#   B  my .dag, main seed: my 04_infer.dag   + main's mirrors           <- measured 6 refusals
#   C  main .dag, my seed: main's 04_infer.dag + my fold carriers       <- the fold's contribution
# NO git checkout / git clean. Everything is restored by cp at the end.
set -uo pipefail
export PATH="/opt/cargo/bin:/usr/local/bin:/usr/local/sbin:$PATH"
cd .
SEED=src/v1/stage0/src
MIRRORS="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
# main's files are STAGED LOCALLY into keep/ and synced by ctrl-build as untracked files: the
# runner checks out only my head and has no 'origin/main' ref, so 'git show origin/main:...' fails
# there (it cost one dispatch).
[ -s keep/main_04_infer.dag ] || { echo "keep/main_04_infer.dag missing -- stage it locally first"; exit 3; }
[ -s keep/my_04_infer.dag ] || cp src/v1/04_infer.dag keep/my_04_infer.dag

arm() { # $1 tag  $2 dag-source  $3 which infer carriers (main|mine)
  # Only the two INFER carriers are ever swapped. emit_rust and the census mirror always stay
  # main's: my carriers were cut at an older base and lack main's new FileVerb::FileWriteBytes
  # match arm, which breaks the lib build (non-exhaustive patterns) -- that is what cost the last
  # dispatch, not a compiler bug.
  local tag=$1 dagsrc=$2 msrc=$3
  cp "$dagsrc" src/v1/04_infer.dag
  for f in $MIRRORS; do
    case "$msrc:$f" in
      main:*) cp "keep/main_$f" "$SEED/$f";;
      mine:v1_compiler_infer.rs|mine:v1_compiler_infer_types.rs) cp "keep/mine_$f" "$SEED/$f";;
      *) cp "keep/main_$f" "$SEED/$f";;
    esac
  done
  echo "########## ARM $tag  dag=$(basename $dagsrc) seed=$msrc  dagrelation=$(grep -c generic_types_unify src/v1/04_infer.dag) seedfold=$(grep -c UnifyGenericsAcc $SEED/v1_compiler_infer.rs)"
  cargo build --release -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|error\[|Finished" | tail -2
  target/release/claim_executor --required-regen --source-root dag --source-root src/v2 > /tmp/rr6_$tag.log 2>&1
  echo "rc=$?  hard-diagnostic-count=$(grep -aoE "produced [0-9]+ hard diagnostic" /tmp/rr6_$tag.log | head -1)"
  echo "--- EVERY diagnostic line, verbatim, uncut:"
  grep -anE "^(type mismatch|call shape|[a-z].*could not resolve|[a-z].*refused)" /tmp/rr6_$tag.log | head -20
  echo "--- (any line mentioning my helpers?) ---"
  grep -cE "generic_types_unify|binding_has_type_variable_leaf|binding_is_unsettled_formal_annotation|generic_head_matches_application" /tmp/rr6_$tag.log
  echo "--- spans named:"
  grep -aoE "\(src/v[12]/[a-z_0-9/]+\.dag:[0-9-]+\)" /tmp/rr6_$tag.log | sort | uniq -c | sort -rn | head -12
  echo "--- first 3 lines after 'refused:' (full width):"
  grep -A3 "refused:" /tmp/rr6_$tag.log | head -8
}
arm A keep/main_04_infer.dag main
arm B keep/my_04_infer.dag main
arm C keep/main_04_infer.dag mine
cp keep/my_04_infer.dag src/v1/04_infer.dag
for f in $MIRRORS; do cp "keep/main_$f" "$SEED/$f"; done
echo "restored: dag=$(sha256sum src/v1/04_infer.dag|cut -c1-12) $(for f in $MIRRORS; do echo -n "$f=$(sha256sum $SEED/$f|cut -c1-10) "; done)"
for t in A B C; do cp /tmp/rr6_$t.log .; done
ls rr6_*.log
echo BISECT6_DONE
