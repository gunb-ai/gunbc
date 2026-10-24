#!/bin/bash
set -u
source <(sed -n '/^B()/p;/^regen()/,/^}/p;/^wit()/,/^}/p' tools/.phaseb_run.sh)
B; regen
wit HEAD
