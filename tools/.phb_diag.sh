#!/bin/bash
set -u
source <(sed -n '/^B()/p;/^regen()/,/^}/p' tools/.phaseb_run.sh)
B; regen
timeout 1800 ./target/release/gunbc run --source-root tools/.phb_rows --source-root dag --entry tools/.phb_rows/p.dag --function main 2>&1 | grep -v "^\[\|user_work_scope" | cut -c1-1500 | tail -40
source <(sed -n '/^wit()/,/^}/p' tools/.phaseb_run.sh)
wit HEAD
