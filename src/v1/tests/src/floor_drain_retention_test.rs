use std::fs;
use std::sync::Mutex;

use crate::helpers::workspace_root;
use v1_compiler::cli_run::{
    build_multi_entry_index, index_retention_snapshot, resolve_entry_with_index,
    typed_cache_evictions_for_test, typed_module_cache_len_for_test,
    typed_module_cache_max_entries,
};

static CAP_ENV_MUTEX: Mutex<()> = Mutex::new(());

fn temp_dir(label: &str) -> std::path::PathBuf {
    let dir = workspace_root()
        .join("target")
        .join(format!("gunbc-floor-drain-{label}-{}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("temp dir");
    dir
}

#[test]
fn typed_module_cache_max_entries_honors_env_override() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "42");
    assert_eq!(typed_module_cache_max_entries(), 42);
    std::env::remove_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES");
}

#[test]
fn index_retention_snapshot_reports_empty_index_shell() {
    let index = build_multi_entry_index(&[]);
    let snap = index_retention_snapshot(&index);
    assert_eq!(snap.typed_module_cache_entries, 0);
    assert_eq!(snap.parse_cache_entries, 0);
    assert_eq!(snap.resolved_graph_memo_entries, 0);
    assert_eq!(snap.typed_cache_evictions, 0);
}

#[test]
fn typed_module_cache_entry_cap_evicts_with_counted_receipt() {
    let _lock = CAP_ENV_MUTEX.lock().expect("cap env mutex");
    let dir = temp_dir("cap");
    let common = "module test.common\n\
        type Box { v: Int }\n\
        fn boxed(n: Int) -> Box { Box { v: n } }\n\
        fn unbox(b: Box) -> Int { b.v }\n";
    let shared1 = "module test.shared1\nfn val() -> Int { 10 }\n";
    let shared2 = "module test.shared2\nfn val() -> Int { 20 }\n";
    let entry_a = "module test.a\n\
        import test.common { boxed, unbox }\n\
        import test.shared1 { val }\n\
        fn witness_a_true() -> Bool { (unbox(boxed(val())) + 0) == 10 }\n";
    let extra = "module test.extra\nfn pad() -> Int { 7 }\n";
    let entry_b = "module test.b\n\
        import test.common { boxed, unbox }\n\
        import test.shared2 { val }\n\
        import test.extra { pad }\n\
        fn witness_b_true() -> Bool { (unbox(boxed(val())) + pad()) == 27 }\n";

    for (name, src) in [
        ("common.dag", common),
        ("shared1.dag", shared1),
        ("shared2.dag", shared2),
        ("extra.dag", extra),
        ("entry_a.dag", entry_a),
        ("entry_b.dag", entry_b),
    ] {
        fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }

    let roots = vec![dir.to_string_lossy().into_owned()];
    let a = dir.join("entry_a.dag").to_string_lossy().into_owned();
    let b = dir.join("entry_b.dag").to_string_lossy().into_owned();

    std::env::set_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES", "2");
    let index = build_multi_entry_index(&roots);
    resolve_entry_with_index(&index, &a).expect("resolve a");
    resolve_entry_with_index(&index, &b).expect("resolve b");

    assert!(
        typed_module_cache_len_for_test(&index) <= 2,
        "typed_module_cache must stay under the modeled cap (got {})",
        typed_module_cache_len_for_test(&index)
    );
    assert!(
        typed_cache_evictions_for_test(&index) > 0,
        "evictions must be counted when the cap is exceeded"
    );

    std::env::remove_var("GUNBC_TYPED_MODULE_CACHE_MAX_ENTRIES");
    let _ = fs::remove_dir_all(&dir);
}
