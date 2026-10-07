mkdir -p /sys/fs/cgroup/gunbcrun && echo 51539607552 > /sys/fs/cgroup/gunbcrun/memory.max && echo $$ > /sys/fs/cgroup/gunbcrun/cgroup.procs
cargo test --release -p v1-compiler --lib --no-run >/dev/null 2>&1
date +%s
cargo test --release -p v1-compiler --lib restore_equals -- --include-ignored 2>&1 | grep -a "test result\|panicked\|FAILED\|ok$"
date +%s
