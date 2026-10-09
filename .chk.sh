git fetch -q --depth=1 origin $1 && git checkout -q -f $1 || { echo FETCHFAIL; exit 3; }
cargo build --release -q -p v1-compiler --bin claim_executor 2>&1 | grep -E "^error|^ *--> " | head -30
