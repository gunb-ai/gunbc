//! Walls for two pre-existing silent seams located 2026-07-04 (both confirmed
//! on pristine main via a worktree A/B; both fixed dag-authority-first):
//!
//! 1. Interpreter kernel-optional unwrap: `match xs |> first { Present {..} }`
//!    over a RECORD element failed non-exhaustive — the raw-payload unwrap
//!    guards were shadowed by the kind-specific match arms (Variant payloads
//!    had an inlined fix; Record/List/Str/Int fell through).
//!
//! 2. Kernel-prelude generic shadowing: a user generic coproduct NAMED
//!    `Optional`, explicitly imported into a multi-import consumer module,
//!    resolved to the paramless KERNEL Optional (ancestry merged kernel as
//!    overlay), so `Optional<T>` signatures were never expanded and pattern
//!    bindings went error-typed with ZERO diagnostics ("error type cascade"
//!    at eval). resolve_generic_use_decl now retries the direct import
//!    parents for a parameterized decl when a type-argument-bearing use site
//!    lands on a paramless one.

use std::fs;

use crate::helpers::workspace_root;
use v1_compiler::cli_run::{make_eval_context, resolve_entry_graph, run_claim, ClaimOutcome};
use v1_compiler::v1_interpreter::ExecutionMode;

fn run_fixture(files: &[(&str, &str)], entry: &str, function: &str) -> ClaimOutcome {
    use std::sync::atomic::{AtomicU64, Ordering};
    static SEQ: AtomicU64 = AtomicU64::new(0);
    let seq = SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = workspace_root()
        .join("target")
        .join(format!("kernel-shadow-seams-{}-{seq}", std::process::id()));
    let _ = fs::remove_dir_all(&dir);
    fs::create_dir_all(&dir).expect("fixture dir");
    for (name, src) in files {
        fs::write(dir.join(name), src).unwrap_or_else(|e| panic!("write {name}: {e}"));
    }
    let roots = vec![
        dir.to_string_lossy().into_owned(),
        workspace_root().join("dag").to_string_lossy().into_owned(),
    ];
    let entry_path = dir.join(entry).to_string_lossy().into_owned();
    let (graph, si) = resolve_entry_graph(&roots, &entry_path).expect("fixture resolve");
    let ctx = make_eval_context(&graph, si, ExecutionMode::Wet);
    let outcome = run_claim(&ctx, function);
    let _ = fs::remove_dir_all(&dir);
    outcome
}

#[test]
fn kernel_optional_record_payload_unwraps_in_match() {
    let src = "module t.seam1\n\
        import std.logic { Bool }\n\
        type Tok { class: Int }\n\
        fn pick(xs: List<Tok>) -> Int {\n\
          match xs |> first {\n\
            Present { value: t } => t.class\n\
            Absent => 0\n\
          }\n\
        }\n\
        test fn seam_one() -> Bool {\n\
          pick(xs: [Tok { class: 3 }]) == 3 && pick(xs: []) == 0\n\
        }\n";
    let outcome = run_fixture(&[("seam1.dag", src)], "seam1.dag", "seam_one");
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "kernel-optional record-payload match must unwrap, got: {outcome:?}"
    );
}

#[test]
fn imported_user_generic_named_optional_shadows_kernel_prelude() {
    let lib = "module t.seamlib\n\
        type Optional<T>\n\
        \x20 = Present { value: T }\n\
        \x20 | Absent\n";
    let entry = "module t.seam2\n\
        import std.logic { Bool }\n\
        import t.seamlib { Optional, Present, Absent }\n\
        type Tok { class: Int }\n\
        fn first_or(xs: List<Tok>) -> Optional<Tok> {\n\
          match xs |> count > 0 {\n\
            true => Present { value: xs |> fold(init: Tok { class: 0 }, f: (acc, t) => t) }\n\
            false => Absent\n\
          }\n\
        }\n\
        type Thunk { apply: fn(List<Tok>) -> Int }\n\
        fn mk() -> Thunk {\n\
          Thunk { apply: fn(toks) {\n\
            match first_or(xs: toks) {\n\
              Present { value: tok } => tok.class\n\
              Absent => 0\n\
            }\n\
          } }\n\
        }\n\
        test fn seam_two() -> Bool {\n\
          let t = mk()\n\
          t.apply([Tok { class: 8 }]) == 8\n\
        }\n";
    let outcome = run_fixture(
        &[("seamlib.dag", lib), ("seam2.dag", entry)],
        "seam2.dag",
        "seam_two",
    );
    assert!(
        matches!(outcome, ClaimOutcome::Pass),
        "imported user generic named Optional must shadow the kernel prelude, got: {outcome:?}"
    );
}
