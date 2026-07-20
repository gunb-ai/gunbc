#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell batch probe orchestrator; until
# then it sequences curated_cargo_probe_one.sh over the four deep-lane targets (04_infer,
# 06_translate, 05_emit family) for baseline TSV receipts.
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe sweep transport in .dag.
# Authority: delegates per-module verdicts to scripts/curated_cargo_probe_one.sh →
# dag/tools/self_host_curated_probe_cargo.dag (no parallel probe spine).
# One-shot deep-module baseline probes (sharp-ferret-568, 2026-07-20).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export CSSL_STD_SEED_LINK=1
PROBE="$ROOT/scripts/curated_cargo_probe_one.sh"
HEAD="$(git rev-parse HEAD)"
HEAD_SHORT="${HEAD:0:12}"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
DATE_TAG="$(date -u +%Y-%m-%d)"
OUT_TSV="${ROOT}/docs/probes/deep_module_baseline_${DATE_TAG}.tsv"

CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc --bin cssl_assemble >/dev/null
GUNBC_HASH="$(sha256sum target/release/gunbc | awk '{print substr($1,1,16)}')"
CSSL_HASH="$(sha256sum target/release/cssl_assemble | awk '{print substr($1,1,16)}')"

# Four deep-lane fan-out targets: 04_infer, 06_translate, 05_emit core + 05_emit family satellites.
DEEP_MODULES=(
  src/v2/compiler/04_infer.dag
  src/v2/compiler/06_translate.dag
  src/v2/compiler/05_emit.dag
  src/v2/compiler/05_emit_orchestration.dag
  src/v2/compiler/emit_host.dag
  src/v2/compiler/emit_module.dag
  src/v2/compiler/emit_produced.dag
  src/v2/compiler/emit_semantic_decl.dag
)

mkdir -p "$(dirname "$OUT_TSV")"
{
  echo "# Deep-module probe baseline (04_infer + 06_translate + 05_emit family)"
  echo "# main_head=$HEAD_SHORT  probed_at=$TS  CSSL_STD_SEED_LINK=1  shim=empty"
  echo "# gunbc_sha256_prefix=$GUNBC_HASH  cssl_assemble_sha256_prefix=$CSSL_HASH"
  echo "# probe: scripts/curated_cargo_probe_one.sh (calm-boar invocation contract)"
  echo "# orchestrator: scripts/run_deep_module_baseline_probes.sh"
  echo -e "module\tbranch_head\tcssl_std_seed_link\tshim_lib_rel\temit\tcargo\tfirst_error\tmapped_gate\tverdict\tprobe_notes"
} >"$OUT_TSV"

for m in "${DEEP_MODULES[@]}"; do
  echo "==> $m" >&2
  row="$("$PROBE" "$m")"
  IFS=$'\t' read -r mod emit cargo err gate verdict <<<"$row"
  case "$mod" in
    src/v2/compiler/04_infer.dag)
      note="LIVE — FreeMonoid-only layer (deep lane 1)"
      ;;
    src/v2/compiler/06_translate.dag)
      note="LIVE — FreeMonoid-only layer (deep lane 2)"
      ;;
    src/v2/compiler/05_emit.dag)
      note="LIVE — FreeMonoid-only layer (deep lane 3 core)"
      ;;
    src/v2/compiler/05_emit_orchestration.dag)
      note="LIVE — FreeMonoid-only layer (deep lane 3 orchestration)"
      ;;
    src/v2/compiler/emit_*.dag)
      note="LIVE — FreeMonoid-only layer (05_emit family)"
      ;;
    *)
      note="LIVE — FreeMonoid-only layer"
      ;;
  esac
  printf '%s\t%s\t1\t\t%s\t%s\t%s\t%s\t%s\t%s\n' \
    "$mod" "$HEAD_SHORT" "$emit" "$cargo" "$err" "$gate" "$verdict" "$note" >>"$OUT_TSV"
done

echo "wrote $OUT_TSV" >&2
