export GUNBC_MEMORY_BUDGET_BYTES=5368709120
cargo build --release -p v1-compiler --bin claim_executor 2>&1 | tail -1
./target/release/claim_executor --required-regen --source-root dag --source-root src/v2 2>&1 | tail -4
echo "===== committed ledger digest line ====="
grep -n "Census identity digest" docs/design-ledgers.md
echo "===== candidate ledger ====="
C=$(find target/stage0-regen-candidate -name 'design-ledgers.md' | head -1)
echo "candidate path: $C"
grep -n "Census identity digest" "$C"
echo "===== full diff committed vs candidate ====="
diff docs/design-ledgers.md "$C" && echo "IDENTICAL"
echo "===== worktree drift ====="
git --no-pager status --short -- docs/
echo "===== done ====="
