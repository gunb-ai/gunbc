#!/usr/bin/env bash
# signals.sh <pin> <path> : print mechanical leads per dimension for one module (dag or rs)
P=$1; F=$2
git show "$P:$F" | grep -nE \
 -e '"/(var|etc|home|tmp|srv|opt|run|usr|root)/[^"]*"' \
 -e '\bsrv[0-9]+\b|fci[0-9]|\.gunb\.ai|localhost|127\.0\.0\.1|0\.0\.0\.0' \
 -e '\b(Filesystem|shell|Shell|Http|http|Git|git|R2|r2|Systemd|systemd|Process|Ssh|ssh)\.[A-Z][A-Za-z]*' \
 -e 'std::fs::|std::env::var|Command::new|env::var|HOME|XDG_' \
 -e '_live_dashboard_instance\(\)' \
 -e 'sudo|sudoers|requires\b|Grant|Capability|token|secret|credential|TOKEN|SECRET' \
 -e 'cache|Cache|memo|Memo|ttl|TTL' \
 -e 'flock|O_EXCL|create_new|rename\(|mkdir|create_dir' \
 -e '\.service"|\.socket"|\.timer"|port: [0-9]+|:[0-9]{4,5}"' \
 -e '^import ' | grep -vE '^\s*[0-9]+:\s*//' 
