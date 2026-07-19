#!/usr/bin/env bash
# SCAFFOLD — dissolve-on: tools.self_host_curated_seed_linked_harness on main post-#6782
# (+ generic std-seed-link follow-up) retires this hand-shell batch probe orchestrator; until
# then it sequences curated_cargo_probe_one.sh over the flip-wave 6 + feeder 13 rosters.
# dissolve-on alt: gunbc bash-emit #5828 / modeled cssl_probe sweep transport in .dag.
# Authority: delegates per-module verdicts to scripts/curated_cargo_probe_one.sh →
# dag/tools/self_host_curated_probe_cargo.dag (no parallel probe spine).
# One-shot flip-wave + 13-tail confirmation probes (neat-swift-795, 2026-07-19).
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"
export CSSL_STD_SEED_LINK=1
PROBE="$ROOT/scripts/curated_cargo_probe_one.sh"
HEAD="$(git rev-parse HEAD)"
HEAD_SHORT="${HEAD:0:12}"
TS="$(date -u +%Y-%m-%dT%H:%M:%SZ)"

CTRL_BUILD_WRAP_CARGO=0 cargo build --release -p v1-compiler --bin gunbc --bin cssl_assemble >/dev/null
GUNBC_HASH="$(sha256sum target/release/gunbc | awk '{print substr($1,1,16)}')"
CSSL_HASH="$(sha256sum target/release/cssl_assemble | awk '{print substr($1,1,16)}')"

probe_modules() {
  local title="$1"
  local out_tsv="$2"
  shift 2
  local -a modules=("$@")
  {
    echo "# $title"
    echo "# main_head=$HEAD_SHORT  probed_at=$TS  CSSL_STD_SEED_LINK=1  shim=empty"
    echo "# gunbc_sha256_prefix=$GUNBC_HASH  cssl_assemble_sha256_prefix=$CSSL_HASH"
    echo "# probe: scripts/curated_cargo_probe_one.sh (calm-boar invocation contract)"
    echo -e "module\tbranch_head\tcssl_std_seed_link\tshim_lib_rel\temit\tcargo\tfirst_error\tmapped_gate\tverdict"
  } >"$out_tsv"
  for m in "${modules[@]}"; do
    echo "==> $m" >&2
    row="$("$PROBE" "$m")"
    IFS=$'\t' read -r mod emit cargo err gate verdict <<<"$row"
    printf '%s\t%s\t1\t\t%s\t%s\t%s\t%s\t%s\n' \
      "$mod" "$HEAD_SHORT" "$emit" "$cargo" "$err" "$gate" "$verdict" >>"$out_tsv"
  done
  echo "wrote $out_tsv" >&2
}

SIX=(
  src/v2/compiler/00_compile.dag
  src/v2/compiler/02_parse.dag
  src/v2/compiler/03_ingest.dag
  src/v2/compiler/materialization_carriers.dag
  src/v2/compiler/program_assembly.dag
  src/v2/compiler/source_authority.dag
)

THIRTEEN=(
  src/v2/compiler/01_tokenize.dag
  src/v2/compiler/04_infer.dag
  src/v2/compiler/program_partition.dag
  src/v2/compiler/05_emit.dag
  src/v2/compiler/05_emit_orchestration.dag
  src/v2/compiler/05_eval.dag
  src/v2/compiler/06_translate.dag
  src/v2/compiler/emit_host.dag
  src/v2/compiler/emit_module.dag
  src/v2/compiler/emit_produced.dag
  src/v2/compiler/emit_semantic_decl.dag
  src/v2/compiler/fold_lowering.dag
  src/v2/compiler/self_host.dag
)

probe_modules \
  "Gate-A flip-wave 6-module re-probe post-#6888+#6886+#6883" \
  "$ROOT/docs/probes/gate_a_flip_wave_6mod_reprobe_2026-07-19.tsv" \
  "${SIX[@]}"

probe_modules \
  "Flip-wave feeder 13-tail confirmation post-#6883 emit-retain (FreeMonoid-only expected)" \
  "$ROOT/docs/probes/flip_wave_feeder_13tail_post_6883_2026-07-19.tsv" \
  "${THIRTEEN[@]}"

echo "DONE probes @ $HEAD_SHORT" >&2
