unset GUNBC_BIND_MEMORY_CGROUP_BYTES
cd src/v1/stage0 && cargo test --release --test generic_slot_substitution_producer 2>&1 | grep -E '^test |test result|panicked|assert' | cut -c1-200
