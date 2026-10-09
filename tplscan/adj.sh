set -e
CG=/sys/fs/cgroup/rg$$; mkdir -p $CG; echo $((40*1024*1024*1024)) > $CG/memory.max; echo $$ > $CG/cgroup.procs
cargo build --release -q --bin gunbc 2>&1 | tail -3
./target/release/gunbc run --claim-run --source-root dag --source-root src/v2 --entry src/v2/test/claim/body_lowering/string_template_test.dag --function emitted_template_tokens_stay_adjacent_so_the_holes_survive_a_re_lex 2>&1 | grep -v "pre-entry\|typecheck" | tail -6
