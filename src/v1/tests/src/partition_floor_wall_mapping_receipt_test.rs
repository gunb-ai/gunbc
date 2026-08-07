//! Execution receipt: map additive partition universe → floor wall for the same CI run.
//!
//! Anchors on GitHub Actions run 31156588100 (main, 2026-08-07) — the same-base control
//! floor whose `[assembly-split]` rows match the lane parent's exclusive partition receipt
//! (7487 witnesses, assembly_rewire_import_str 40.3%, assembly_symbol_index 22.6%, …).
//!
//! Reports descriptive ratios only (`row_seconds / floor_wall`, `row_seconds / discovery_wall`,
//! `resolve_seconds / floor_wall`). Realized floor savings require a measured A/B — no causal
//! multiplier from discovery_wall / resolve_serial.

use std::collections::BTreeMap;

/// Parsed inputs from CI floor log lines + GitHub Actions step timing.
struct FloorRunAnchors {
    github_run_id: u64,
    floor_step_wall_seconds: u64,
    governor_target_width: u64,
    witness_rows: u64,
    resolve_serial_seconds: f64,
    eval_serial_seconds: f64,
    discovery_wall_seconds: f64,
    ordinary_worker_elapsed_at_discovery_end_seconds: u64,
    assembly_split_ms: BTreeMap<String, f64>,
    resolve_split_ms: BTreeMap<String, f64>,
}

fn anchors_ci_run_31156588100() -> FloorRunAnchors {
    let mut assembly_split_ms = BTreeMap::new();
    assembly_split_ms.insert("schedule".into(), 573.5);
    assembly_split_ms.insert("probe".into(), 1484.3);
    assembly_split_ms.insert("graph".into(), 527.8);
    assembly_split_ms.insert("symbol_index".into(), 180332.1);
    assembly_split_ms.insert("pool_fill".into(), 0.3);
    assembly_split_ms.insert("symbol_index_merge".into(), 10925.4);
    assembly_split_ms.insert("variant_base".into(), 10619.5);
    assembly_split_ms.insert("root_symbol_index".into(), 5986.9);
    assembly_split_ms.insert("root_variant_base".into(), 51830.6);
    assembly_split_ms.insert("environment".into(), 7181.0);
    assembly_split_ms.insert("diagnostics".into(), 102.4);
    assembly_split_ms.insert("registry".into(), 1187.5);
    assembly_split_ms.insert("services".into(), 57412.8);
    assembly_split_ms.insert("rewire_type_env".into(), 13479.5);
    assembly_split_ms.insert("rewire_import_str".into(), 322082.7);
    assembly_split_ms.insert("rewire_func_env".into(), 5938.7);
    assembly_split_ms.insert("emit_info".into(), 55324.2);
    assembly_split_ms.insert("other".into(), 5446.1);

    let mut resolve_split_ms = BTreeMap::new();
    resolve_split_ms.insert("load".into(), 2.9);
    resolve_split_ms.insert("parse".into(), 5974.8);
    resolve_split_ms.insert("resolve".into(), 19170.6);
    resolve_split_ms.insert("normalize".into(), 694.3);
    resolve_split_ms.insert("typecheck".into(), 40971.1);
    resolve_split_ms.insert("parent_envs".into(), 7.4);
    resolve_split_ms.insert("reconcile_assembly".into(), 5446.1);
    resolve_split_ms.insert("ownership".into(), 2097.6);
    resolve_split_ms.insert("other".into(), 3080.5);

    FloorRunAnchors {
        github_run_id: 31156588100,
        // GitHub Actions step "gunbc ci (.dag witnesses + gates)" wall clock.
        floor_step_wall_seconds: 3290,
        governor_target_width: 1,
        witness_rows: 7487,
        resolve_serial_seconds: 802434.506 / 1000.0,
        eval_serial_seconds: 96820.309 / 1000.0,
        // Discovery pump wall: run_discovery_corpus start → [measurement] discovery corpus line.
        discovery_wall_seconds: 1016.0,
        ordinary_worker_elapsed_at_discovery_end_seconds: 2190,
        assembly_split_ms,
        resolve_split_ms,
    }
}

fn sum_ms(map: &BTreeMap<String, f64>) -> f64 {
    map.values().sum()
}

/// Descriptive mapping — ratios only, no causal conversion factor.
struct PartitionFloorMapping {
    partition_resolve_serial_seconds: f64,
    partition_split_sum_seconds: f64,
    floor_step_wall_seconds: u64,
    discovery_wall_seconds: f64,
    /// resolve_serial / discovery_wall (descriptive, not causal).
    resolve_serial_per_discovery_wall: f64,
    /// resolve_serial / floor_step_wall.
    partition_coverage_of_floor: f64,
    /// discovery_wall / floor_step_wall.
    discovery_share_of_floor: f64,
}

fn derive_mapping(a: &FloorRunAnchors) -> PartitionFloorMapping {
    let partition_split_sum_seconds =
        (sum_ms(&a.assembly_split_ms) + sum_ms(&a.resolve_split_ms)) / 1000.0;
    let resolve_serial = a.resolve_serial_seconds;
    let discovery_wall = a.discovery_wall_seconds;
    let floor_wall = a.floor_step_wall_seconds as f64;

    PartitionFloorMapping {
        partition_resolve_serial_seconds: resolve_serial,
        partition_split_sum_seconds,
        floor_step_wall_seconds: a.floor_step_wall_seconds,
        discovery_wall_seconds: discovery_wall,
        resolve_serial_per_discovery_wall: resolve_serial / discovery_wall,
        partition_coverage_of_floor: resolve_serial / floor_wall,
        discovery_share_of_floor: discovery_wall / floor_wall,
    }
}

#[test]
fn partition_floor_wall_mapping_receipt() {
    let a = anchors_ci_run_31156588100();
    let m = derive_mapping(&a);

    // Parent partition universe ≈ 799s; measurement resolve serial is the emitted anchor.
    assert!(
        m.partition_resolve_serial_seconds > 795.0 && m.partition_resolve_serial_seconds < 805.0,
        "resolve serial {}s should match ~799s partition universe",
        m.partition_resolve_serial_seconds
    );

    // Lane parent partition row shares (same run).
    let rewire_import_str_s = a.assembly_split_ms["rewire_import_str"] / 1000.0;
    let symbol_index_s = a.assembly_split_ms["symbol_index"] / 1000.0;
    let parse_s = a.resolve_split_ms["parse"] / 1000.0;
    let typecheck_s = a.resolve_split_ms["typecheck"] / 1000.0;

    let rewire_share = rewire_import_str_s / m.partition_resolve_serial_seconds;
    let symbol_index_share = symbol_index_s / m.partition_resolve_serial_seconds;
    let parse_typecheck_share = (parse_s + typecheck_s) / m.partition_resolve_serial_seconds;

    assert!(
        rewire_share > 0.39 && rewire_share < 0.41,
        "rewire_import_str share {}",
        rewire_share
    );
    assert!(
        symbol_index_share > 0.22 && symbol_index_share < 0.24,
        "symbol_index share {}",
        symbol_index_share
    );
    assert!(
        parse_typecheck_share > 0.055 && parse_typecheck_share < 0.065,
        "parse+typecheck share {}",
        parse_typecheck_share
    );

    // Width=1 on this run: no cross-worker shard parallelism inside discovery.
    assert_eq!(a.governor_target_width, 1);

    // Coverage: partition resolve serial is NOT the whole floor wall.
    assert!(
        m.partition_coverage_of_floor > 0.23 && m.partition_coverage_of_floor < 0.27,
        "partition should cover ~24% of floor wall, got {}",
        m.partition_coverage_of_floor
    );

    // Row shares of floor wall (descriptive only — no multiplier substitution).
    let symbol_index_share_of_floor = symbol_index_s / (m.floor_step_wall_seconds as f64);
    let parse_typecheck_share_of_floor =
        (parse_s + typecheck_s) / (m.floor_step_wall_seconds as f64);

    eprintln!(
    "[partition-floor-mapping] run={} floor_step_wall_s={} discovery_wall_s={} resolve_serial_s={:.3} split_sum_s={:.1} width={}",
    a.github_run_id,
    m.floor_step_wall_seconds,
    m.discovery_wall_seconds,
    m.partition_resolve_serial_seconds,
    m.partition_split_sum_seconds,
    a.governor_target_width,
  );
    eprintln!(
    "[partition-floor-mapping] partition_coverage_of_floor={:.3} discovery_share_of_floor={:.3} resolve_per_discovery_wall={:.3} (ratio only)",
    m.partition_coverage_of_floor,
    m.discovery_share_of_floor,
    m.resolve_serial_per_discovery_wall,
  );
    eprintln!(
    "[partition-floor-mapping] row_shares_of_resolve_serial: rewire_import_str={:.1}% symbol_index={:.1}% parse+typecheck={:.1}%",
    rewire_share * 100.0,
    symbol_index_share * 100.0,
    parse_typecheck_share * 100.0,
  );
    eprintln!(
    "[partition-floor-mapping] row_shares_of_floor_wall: symbol_index={:.3}% parse+typecheck={:.3}% (descriptive — realized savings need A/B)",
    symbol_index_share_of_floor * 100.0,
    parse_typecheck_share_of_floor * 100.0,
  );
    eprintln!(
    "[partition-floor-mapping] NOT CAUSAL: discovery_wall/resolve_serial={:.3} does not convert partition deltas to floor seconds",
    m.discovery_wall_seconds / m.partition_resolve_serial_seconds,
  );

    // Sanity: parse+typecheck is a small share of total floor wall on this run.
    assert!(
        parse_typecheck_share_of_floor < 0.10,
        "parse+typecheck should be <10% of floor wall"
    );

    // symbol_index is under 8% of floor wall on this run.
    assert!(
        symbol_index_share_of_floor < 0.08,
        "symbol_index should be <8% of floor wall, got {}",
        symbol_index_share_of_floor
    );
}
