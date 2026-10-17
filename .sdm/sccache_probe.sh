echo "SCCACHE-PROBE $(date -u +%H:%M:%S) host=$(hostname)"
env | grep -i "sccache\|RUSTC_WRAPPER" || echo "(no sccache env)"
sccache --show-stats 2>&1 | head -10
echo PROBE_DONE
