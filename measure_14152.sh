#!/usr/bin/env bash
# Measurement for #14152, review 81061 items (1) and (2). One remote dispatch.
#
# VARIABLE UNDER TEST: the SHIPPED SEED's execution of the unify_generics fold.
#   GREEN seed = 2751b200 (ci auto-heal regeneration) -> seedgreen/
#   RED   seed = fa06f6b45ca (pre-regen, binds first-wins) -> seedred/
# Seeds ship as FILES. The runner mirrors an uncommitted tree whose git base is not this
# PR's head, so `git checkout <ref> -- path` writes the base's content instead, and
# untracked helper files are wiped by `git clean` between dispatches.
#
# Everything runs through `cargo test` on the shipped seed crate. claim_batch is not
# usable on the runner: the host budget refuses to be absorbed (HostBudgetUnreadable,
# cli_run/entry_resolve.rs; a fabricated GUNBC_MEMORY_BUDGET_BYTES is ruled out by
# dag/gunbc/host/host_budget_source.dag) and the runner grants no cgroup write (EPERM on
# both cgroup.subtree_control and mkdir under /sys/fs/cgroup), so the ci_spec.dag bind
# recipe is unavailable there. That refusal is not worked around.
set -u
SEED=src/v1/stage0/src
FILES="v1_compiler_infer.rs v1_compiler_infer_types.rs v1_compiler_emit_rust.rs v1_tests_claim_generic_identity_census.rs"
sig() { grep -c 'UnifyGenericsAcc' "$SEED/v1_compiler_infer.rs"; }
shax() { sha256sum "$SEED/v1_compiler_infer.rs" | cut -c1-12; }
put() { for f in $FILES; do cp "$1/$f" "$SEED/$f"; done; }

report() { # $1 log
  grep -E "^test cross_coproduct|test result" "$1" | sed 's/ \.\.\. ◐ started compile.frontend//' | sed 's/^test //; s/ \.\.\. / => /' | tail -20
  echo "--- refusal text seen in panics (first 120 chars each):"
  grep -E "must refuse|must not refuse" "$1" | cut -c1-120 | head -8
  echo "--- compile errors:"; grep -E "^error" "$1" | head -6
}

echo "HEAD=$(git rev-parse HEAD)"
echo "EXPECTED GREEN sha=5e9c37585fe0 (sig 36)  EXPECTED RED sha=$(shax) with sig 0"

for side in seedgreen seedred; do
  echo "########## $side : sig=$( [ $side = seedgreen ] && echo 36 || echo 0 ) ##########"
  put $side; echo "APPLIED sig=$(sig) sha=$(shax)"
  cargo test -p v1-compiler-tests --lib cross_coproduct > /tmp/m_$side.log 2>&1
  echo "EXIT[$side]=$?"
  report /tmp/m_$side.log
done

echo "########## SEED RESTORED TO HEAD ##########"
put seedgreen; echo "RESTORED sig=$(sig) sha=$(shax)"
echo "########## DONE ##########"
