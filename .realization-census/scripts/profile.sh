#!/usr/bin/env bash
# profile.sh <pin> <path> : comment-stripped mechanical profile of one module
P=$1; F=$2; T=$(mktemp); git show "$P:$F" | grep -vE '^\s*//' > $T
echo "## $F ($(wc -l <$T) code lines)"
echo "ambient: $(grep -oE '\b(Filesystem|shell|Http|Git|R2|Systemd|Process|Ssh)\.[A-Z][A-Za-z.]*|std::fs::[a-z_]+|Command::new|std::env::var|env::var' $T | sort | uniq -c | sort -rn | tr -s ' ' | tr '\n' ';')"
echo "live_pins: $(grep -c '_live_dashboard_instance()' $T)"
echo "env: $(grep -oE '"[A-Z][A-Z0-9_]{3,}"' $T | sort -u | tr '\n' ' ')"
echo "abs_paths: $(grep -oE '"/(var|etc|home|tmp|srv|opt|run|usr|root|nix|mnt)/[^"]*"' $T | sort -u | tr '\n' ' ')"
echo "hosts: $(grep -oE '"[^"]*(srv[0-9]|fci[0-9]|\.gunb\.ai|localhost|127\.0\.0\.1|r2\.cloudflarestorage|amazonaws)[^"]*"' $T | sort -u | head -12 | tr '\n' ' ')"
echo "auth_imports: $(grep -oE '^import (std\.(artifact_store|cache_interface|durable_compare_and_set|realization|materialization_[a-z]+|resources|scoped_authorization|effect_grant)|gunbc\.(durable_cas_file_store|filesystem_interface|owned_directory|managed_directory|roadmap\.dashboard_instance_directories|roadmap\.roadmap_dashboard_instance|auth\.[a-z_]+|compute\.work_provider_local|live_deploy\.[a-z_]+|fleet\.[a-z_]+)|product\.fabric\.selection|extdeps\.realization\.cache_purity)' $T | sed 's/import //' | sort -u | tr '\n' ' ')"
echo "mech: $(grep -oiE 'flock|O_EXCL|create_new|WriteCreateNew|Mkdir[A-Za-z.]*|create_dir_all|rename|memo[a-z_]*|cache[a-z_]*|HashMap<|OnceLock|lazy_static|thread_local|sudo[a-z]*|ttl' $T | sort | uniq -c | sort -rn | head -12 | tr -s ' ' | tr '\n' ';')"
rm $T
