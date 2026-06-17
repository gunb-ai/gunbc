// Hand-maintained instrumentation for interface-elaboration localization (merry-crab brief).
// Enable with GUNBC_INSTRUMENT_INTERFACE_ELABORATION=1.

use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{Duration, Instant};

#[derive(Default, Clone)]
struct ImportMergeStats {
    calls: u64,
    total: Duration,
    max: Duration,
}

thread_local! {
    static ENABLED: RefCell<bool> = const { RefCell::new(false) };
    static TYPECHECK_TOTAL: RefCell<HashMap<String, Duration>> = RefCell::new(HashMap::new());
    static BUILD_ENV_TOTAL: RefCell<HashMap<String, Duration>> = RefCell::new(HashMap::new());
    static IMPORT_MERGE: RefCell<HashMap<String, ImportMergeStats>> = RefCell::new(HashMap::new());
    static MERGE_SCOPE_TOTAL: RefCell<HashMap<String, Duration>> = RefCell::new(HashMap::new());
}

static ENV_CHECKED: AtomicBool = AtomicBool::new(false);

fn ensure_env_checked() {
    if ENV_CHECKED.swap(true, Ordering::Relaxed) {
        return;
    }
    if std::env::var("GUNBC_INSTRUMENT_INTERFACE_ELABORATION")
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
    {
        ENABLED.with(|f| *f.borrow_mut() = true);
    }
}

pub fn enabled() -> bool {
    ensure_env_checked();
    ENABLED.with(|f| *f.borrow())
}

pub fn reset() {
    TYPECHECK_TOTAL.with(|m| m.borrow_mut().clear());
    BUILD_ENV_TOTAL.with(|m| m.borrow_mut().clear());
    IMPORT_MERGE.with(|m| m.borrow_mut().clear());
    MERGE_SCOPE_TOTAL.with(|m| m.borrow_mut().clear());
}

pub struct ElabGuard {
    map: &'static std::thread::LocalKey<RefCell<HashMap<String, Duration>>>,
    key: String,
    start: Instant,
}

impl ElabGuard {
    pub fn new(
        map: &'static std::thread::LocalKey<RefCell<HashMap<String, Duration>>>,
        key: String,
    ) -> Option<Self> {
        if !enabled() {
            return None;
        }
        Some(Self {
            map,
            key,
            start: Instant::now(),
        })
    }
}

impl Drop for ElabGuard {
    fn drop(&mut self) {
        let elapsed = self.start.elapsed();
        self.map.with(|m| {
            *m.borrow_mut().entry(self.key.clone()).or_insert(Duration::ZERO) += elapsed;
        });
    }
}

pub fn record_import_merge(consumer: &str, import_path: &str, elapsed: Duration) {
    if !enabled() {
        return;
    }
    let key = format!("{consumer} <- {import_path}");
    IMPORT_MERGE.with(|m| {
        let entry = m.borrow_mut().entry(key).or_default();
        entry.calls += 1;
        entry.total += elapsed;
        if elapsed > entry.max {
            entry.max = elapsed;
        }
    });
}

pub fn eprint_report(context: &str) {
    if !enabled() {
        return;
    }
    eprintln!("[interface-elab] {context}");

    fn print_duration_map(label: &str, map: &HashMap<String, Duration>) {
        if map.is_empty() {
            return;
        }
        let mut rows: Vec<_> = map.iter().collect();
        rows.sort_by(|a, b| b.1.cmp(a.1));
        eprintln!("  {label}:");
        for (name, dur) in rows.iter().take(20) {
            eprintln!("    {name}: {dur:?}");
        }
    }

    TYPECHECK_TOTAL.with(|m| print_duration_map("typecheck_module", &m.borrow()));
    BUILD_ENV_TOTAL.with(|m| print_duration_map("build_type_env", &m.borrow()));
    MERGE_SCOPE_TOTAL.with(|m| print_duration_map("merge_scope_from_imports", &m.borrow()));

    IMPORT_MERGE.with(|m| {
        let map = m.borrow();
        if map.is_empty() {
            return;
        }
        let mut rows: Vec<_> = map.iter().collect();
        rows.sort_by(|a, b| b.1.total.cmp(&a.1.total));
        eprintln!("  build_type_env import_bindings merge:");
        for (key, stats) in rows.iter().take(20) {
            eprintln!(
                "    {key}: total={:?} calls={} max={:?}",
                stats.total, stats.calls, stats.max
            );
        }
    });
}

pub fn typecheck_guard(module_name: &str) -> Option<ElabGuard> {
    ElabGuard::new(&TYPECHECK_TOTAL, module_name.to_string())
}

pub fn build_env_guard(module_name: &str) -> Option<ElabGuard> {
    ElabGuard::new(&BUILD_ENV_TOTAL, module_name.to_string())
}

pub fn merge_scope_guard(consumer: &str) -> Option<ElabGuard> {
    ElabGuard::new(&MERGE_SCOPE_TOTAL, consumer.to_string())
}
