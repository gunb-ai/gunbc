//! NOMINAL-INHABITANCE HOLE, RED-first (tidy-ant-251's brief, 2026-10-20).
//!
//! DEFECT: the front end ACCEPTS a value of one nominal record type at a slot
//! whose declared type is a DIFFERENT, field-isomorphic nominal type, because
//! the judgment that fires compares STRUCTURE (field labels and types) where
//! the declaration's IDENTITY decides. Real producer:
//! v2.compiler.reference_conservation `refusal_cover_add` snocs the literal
//! `ByteRange { start: s, end: e }` (`ByteRange`: a variant of the Locus
//! Extent coproduct, v2.std.diagnostic:10) into
//! `ranges: FreeMonoid<RefusalRange>` (`RefusalRange`: a local record with the
//! identical field shape, reference_conservation:511) — rustc is the FIRST
//! refusal (E0308 at the emitted v2_compiler_reference_conservation.rs:375).
//!
//! MEASURED MATRIX (runner, linux/amd64 16CPU/64GB, GUNBC_NATIVE_PRODUCT_REUSE=fresh,
//! SHA 2040046d44a, `gunbc run --claim-run` — the production resolve+infer route):
//! - ACCEPTED (invalid state): field-isomorphic value at a GENERIC call formal of a
//!   monomorphic container field (the reference_conservation shape, real
//!   `std.algebra.list_snoc_item` + imported `ByteRange`); the same shape with a
//!   locally declared generic `snoc<T>` and either a local or imported sibling;
//!   and a two-argument generic call where the two actuals are different nominal
//!   records unified through one type variable. All evaluate.
//! - REFUSED (working guards): a record literal at a DIRECT (non-generic) slot
//!   declared as a different record ("value does not inhabit its declared type
//!   at the direct call argument ... declared 'Product(RefusalRange)', produced
//!   'Product(OtherRange)'"); a coproduct variant at a direct record slot; a
//!   record at an `Int` slot. The accept is specific to the generic-formal path,
//!   NOT systematic.
//!
//! These tests pin the pair M1 asked for on the production CLI route: a
//! field-isomorphic sibling at a generic formal of a monomorphic container field
//! must REFUSE with a typed cause naming BOTH declarations, while the same type
//! at the same slot ACCEPTS. On main the mismatch arms are RED (the run is
//! accepted and evaluates); the controls are GREEN and guard the fix against
//! blunt-force regressions. Each arm is its OWN entry file: refusal is per
//! entry closure, and one closure must not carry an arm it is not testing.
//!
//! RECURRING FAILURE MODE ROW:
//! dag/gunbc/recurring_failure_mode/nominal_isomorphic_record_admitted_at_sibling_declaration.dag
//!
//! HAND-WRITTEN-RUST RECEIPT (ctrl review policy; reviews 83758/83778/83935
//! advisories). DEFERRAL, naming a lane and a tracking row: this file is a
//! ROUTE harness, not seed behavior -- it shells the production `gunbc`
//! (`CARGO_BIN_EXE_gunbc run --claim-run`) so the judgment is observed at the
//! CLI route rather than in-process. Seed `.dag` cannot express spawning the
//! real binary, so no deleted scaffold could replace it and a seed census of
//! this file is not meaningful; the before/after census it asks for IS the
//! arm roster measured here (main: mismatch arms RED `3 passed; 2 failed`;
//! head: `8 passed; 0 failed`), and it is tracked by the RFM row named above,
//! in the v1 stage0 test lane (same route idiom as scoped_run_observation.rs,
//! the pre-existing production-CLI-route test in this directory).

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicUsize, Ordering};

static FIXTURE_ID: AtomicUsize = AtomicUsize::new(0);

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .expect("repo root")
}

fn fixture_root(label: &str) -> PathBuf {
    let root = repo_root().join("target").join(format!(
        "gunbc-nominal-admission-{}-{}-{}",
        label,
        std::process::id(),
        FIXTURE_ID.fetch_add(1, Ordering::Relaxed)
    ));
    std::fs::create_dir_all(&root).expect("create fixture root");
    root
}

const PREAMBLE: &str = "module test.fixtures.nominal_isomorphic_record_admission\n\
     \n\
     import std.algebra { Cons, Empty, FreeMonoid, length, list_snoc_item }\n\
     import v2.std.diagnostic { ByteRange }\n\
     \n\
     type ProcessExit = ExitSuccess | ExitFailure { code: Int, reason: String }\n\
     \n\
     type RefusalRange {\n\
     \x20 start: Int\n\
     \x20 end: Int\n\
     }\n\
     \n\
     type RefusalCover {\n\
     \x20 anchored: Int\n\
     \x20 ranges: FreeMonoid<RefusalRange>\n\
     }\n";

/// REAL PRODUCER SHAPE: `ByteRange` (the Locus Extent coproduct variant, a
/// field-isomorphic sibling of `RefusalRange`) at `FreeMonoid<RefusalRange>`
/// through the production `std.algebra.list_snoc_item` — the refusal_cover_add
/// expression, verbatim in kind. Must REFUSE; on main it is ACCEPTED.
#[test]
fn field_isomorphic_sibling_at_generic_slot_refuses_naming_both_declarations() {
    let root = fixture_root("real-mismatch");
    let entry = root.join("nominal_real_mismatch.dag");
    std::fs::write(
        &entry,
        format!(
            "{PREAMBLE}\n\
             fn probe() -> ProcessExit {{\n\
             \x20 let c = RefusalCover {{ anchored: 0, ranges: Empty }}\n\
             \x20 let g = list_snoc_item(xs: c.ranges, item: ByteRange {{ start: 1, end: 2 }})\n\
             \x20 if length(xs: g) == 1 {{ ExitSuccess }} else {{ ExitFailure {{ code: 1, reason: \"unexpected\" }} }}\n\
             }}\n"
        ),
    )
    .expect("write real mismatch fixture");
    let output = run_fixture(&root, &entry, "probe");
    let _ = std::fs::remove_dir_all(&root);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        !output.status.success(),
        "a ByteRange (v2.std.diagnostic Extent variant) at FreeMonoid<RefusalRange> \
         must not be accepted; the run exited {} with stderr:\n{stderr}",
        output.status,
    );
    // Measured cause (post-fix): "value does not inhabit its declared type at the
    // direct call argument for parameter 'item' [RefusedProductCoproductHeadMismatch]:
    // declared 'Product(RefusalRange)', produced 'Coproduct(Extent)'" -- the typed
    // cause names the product declaration and the coproduct the ByteRange arm belongs to.
    assert!(
        stderr.contains("does not inhabit its declared type")
            && stderr.contains("RefusalRange")
            && (stderr.contains("ByteRange") || stderr.contains("Extent")),
        "the refusal must be the typed inhabitance cause naming BOTH declarations \
         (RefusalRange and the ByteRange/Extent side), got:\n{stderr}"
    );
}

/// POSITIVE CONTROL (green on main, must stay green): the same type at the
/// same generic slot is ordinary inhabitance and ACCEPTS.
#[test]
fn same_nominal_at_generic_slot_is_accepted() {
    let root = fixture_root("real-control");
    let entry = root.join("nominal_real_control.dag");
    std::fs::write(
        &entry,
        format!(
            "{PREAMBLE}\n\
             fn probe() -> ProcessExit {{\n\
             \x20 let c = RefusalCover {{ anchored: 0, ranges: Empty }}\n\
             \x20 let g = list_snoc_item(xs: c.ranges, item: RefusalRange {{ start: 1, end: 2 }})\n\
             \x20 if length(xs: g) == 1 {{ ExitSuccess }} else {{ ExitFailure {{ code: 1, reason: \"control shape broken\" }} }}\n\
             }}\n"
        ),
    )
    .expect("write real control fixture");
    let output = run_fixture(&root, &entry, "probe");
    let _ = std::fs::remove_dir_all(&root);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "the same-nominal control must be accepted, got exit {}:\n{stderr}",
        output.status,
    );
}

// The local pair mirrors the measured ACCEPT shape (probe P2, j24270644:
// `returned \`2\``): std.algebra's FreeMonoid with a LOCALLY declared generic
// snoc<T>, two actuals unified through one type variable. Earlier in-test
// variants over a locally declared Mono<T> coproduct with a match readout
// refused — a different judgment fires there — so the arms keep the probe's
// readout (length) and container.
const LOCAL_PREAMBLE: &str = "module test.fixtures.nominal_isomorphic_record_local\n\
     \n\
     import std.algebra { Cons, Empty, FreeMonoid, length }\n\
     \n\
     type ProcessExit = ExitSuccess | ExitFailure { code: Int, reason: String }\n\
     \n\
     type RefusalRange {\n\
     \x20 start: Int\n\
     \x20 end: Int\n\
     }\n\
     \n\
     type OtherRange {\n\
     \x20 start: Int\n\
     \x20 end: Int\n\
     }\n\
     \n\
     fn local_snoc<T>(xs: FreeMonoid<T>, item: T) -> FreeMonoid<T> {\n\
     \x20 Cons { head: item, tail: xs }\n\
     }\n";

/// THE RED, LOCAL CASE: the accept is the generic-formal path itself, not
/// std.algebra or an import boundary. On main this FAILS (measured `returned
/// \`2\``, accepted; j24270644 P2).
#[test]
fn local_record_sibling_at_local_generic_slot_refuses_naming_both_declarations() {
    let root = fixture_root("local-mismatch");
    let entry = root.join("nominal_local_mismatch.dag");
    std::fs::write(
        &entry,
        format!(
            "{LOCAL_PREAMBLE}\n\
             fn probe() -> ProcessExit {{\n\
             \x20 let g = local_snoc(xs: Empty, item: OtherRange {{ start: 1, end: 2 }})\n\
             \x20 let h = local_snoc(xs: g, item: RefusalRange {{ start: 3, end: 4 }})\n\
             \x20 if length(xs: h) == 2 {{ ExitSuccess }} else {{ ExitFailure {{ code: 1, reason: \"unexpected\" }} }}\n\
             }}\n"
        ),
    )
    .expect("write local mismatch fixture");
    let output = run_fixture(&root, &entry, "probe");
    let _ = std::fs::remove_dir_all(&root);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        !output.status.success(),
        "an OtherRange unified with a RefusalRange through one type variable of a \
         locally declared snoc<T> must not be accepted; the run exited {} with stderr:\n{stderr}",
        output.status,
    );
    assert!(
        stderr.contains("does not inhabit its declared type")
            && stderr.contains("RefusalRange")
            && stderr.contains("OtherRange"),
        "the refusal must name BOTH declarations (RefusalRange and OtherRange), got:\n{stderr}"
    );
}

/// POSITIVE CONTROL, LOCAL CASE (green on main, must stay green): same type at
/// the same local generic slot ACCEPTS.
#[test]
fn local_same_nominal_at_local_generic_slot_is_accepted() {
    let root = fixture_root("local-control");
    let entry = root.join("nominal_local_control.dag");
    std::fs::write(
        &entry,
        format!(
            "{LOCAL_PREAMBLE}\n\
             fn probe() -> ProcessExit {{\n\
             \x20 let g = local_snoc(xs: Empty, item: RefusalRange {{ start: 1, end: 2 }})\n\
             \x20 let h = local_snoc(xs: g, item: RefusalRange {{ start: 3, end: 4 }})\n\
             \x20 if length(xs: h) == 2 {{ ExitSuccess }} else {{ ExitFailure {{ code: 1, reason: \"control shape broken\" }} }}\n\
             }}\n"
        ),
    )
    .expect("write local control fixture");
    let output = run_fixture(&root, &entry, "probe");
    let _ = std::fs::remove_dir_all(&root);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "the local same-nominal control must be accepted, got exit {}:\n{stderr}",
        output.status,
    );
}

/// GUARD CONTROL (green on main, must stay green): the direct (non-generic)
/// slot already refuses with the typed cause; the fix must not weaken it.
#[test]
fn direct_record_sibling_at_direct_slot_still_refuses() {
    let root = fixture_root("direct-guard");
    let entry = root.join("nominal_direct_guard.dag");
    std::fs::write(
        &entry,
        format!(
            "{LOCAL_PREAMBLE}\n\
             fn takes_range(r: RefusalRange) -> Int {{\n\
             \x20 r.start\n\
             }}\n\
             \n\
             fn probe() -> ProcessExit {{\n\
             \x20 let n = takes_range(r: OtherRange {{ start: 1, end: 2 }})\n\
             \x20 if n == 1 {{ ExitSuccess }} else {{ ExitFailure {{ code: 1, reason: \"guard control shape broken\" }} }}\n\
             }}\n"
        ),
    )
    .expect("write direct guard fixture");
    let output = run_fixture(&root, &entry, "probe");
    let _ = std::fs::remove_dir_all(&root);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        !output.status.success(),
        "the direct-slot guard must keep refusing, got exit {}:\n{stderr}",
        output.status,
    );
    assert!(
        stderr.contains("does not inhabit its declared type")
            && stderr.contains("RefusalRange")
            && stderr.contains("OtherRange"),
        "the direct refusal must name BOTH declarations, got:\n{stderr}"
    );
}

/// LEGITIMATE MULTI-INSTANTIATION CONTROL (M1 condition 3, must never refuse):
/// the same generic fn called at two different T in one module. Each call is
/// judged against its OWN substituted formal; different instantiations of one
/// generic are ordinary inhabitance.
#[test]
fn same_generic_fn_at_two_different_instantiations_is_accepted() {
    let root = fixture_root("multi-instantiation");
    let entry = root.join("nominal_multi_instantiation.dag");
    std::fs::write(
        &entry,
        format!(
            "{LOCAL_PREAMBLE}\n\
             fn identity<T>(x: T) -> T {{\n\
             \x20 x\n\
             }}\n\
             \n\
             fn probe() -> ProcessExit {{\n\
             \x20 let a = identity(x: RefusalRange {{ start: 1, end: 2 }})\n\
             \x20 let b = identity(x: OtherRange {{ start: 3, end: 4 }})\n\
             \x20 if a.start + b.start == 4 {{ ExitSuccess }} else {{ ExitFailure {{ code: 1, reason: \"control shape broken\" }} }}\n\
             }}\n"
        ),
    )
    .expect("write multi-instantiation fixture");
    let output = run_fixture(&root, &entry, "probe");
    let _ = std::fs::remove_dir_all(&root);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "the same generic fn at two different T in one module must be accepted, got exit {}:\n{stderr}",
        output.status,
    );
}

/// COPRODUCT-ARM CONTROL (M1 condition 3, must never refuse): a T bound to a
/// coproduct, judged with the coproduct's arm as the argument — both the arm
/// literal at a T slot (T := arm head) and a value statically typed as the
/// coproduct parent at a T slot (T := parent).
#[test]
fn generic_slot_at_coproduct_and_its_arm_is_accepted() {
    let root = fixture_root("coproduct-arm");
    let entry = root.join("nominal_coproduct_arm.dag");
    std::fs::write(
        &entry,
        format!(
            "{LOCAL_PREAMBLE}\n\
             type Shape\n\
             \x20 = Circle {{ r: Int }}\n\
             \x20 | Square {{ side: Int }}\n\
             \n\
             fn identity<T>(x: T) -> T {{\n\
             \x20 x\n\
             }}\n\
             \n\
             fn probe() -> ProcessExit {{\n\
             \x20 let a = identity(x: Circle {{ r: 3 }})\n\
             \x20 let b: Shape = Square {{ side: 4 }}\n\
             \x20 let c = identity(x: b)\n\
             \x20 match a {{\n\
             \x20\x20 Circle {{ r: r }} =>\n\
             \x20\x20\x20 match c {{\n\
             \x20\x20\x20\x20 Circle {{ r: r2 }} => ExitFailure {{ code: 1, reason: \"control shape broken\" }}\n\
             \x20\x20\x20\x20 Square {{ side: s }} => if r + s == 7 {{ ExitSuccess }} else {{ ExitFailure {{ code: 1, reason: \"control shape broken\" }} }}\n\
             \x20\x20\x20 }}\n\
             \x20\x20 Square {{ side: _ }} => ExitFailure {{ code: 1, reason: \"control shape broken\" }}\n\
             \x20 }}\n\
             }}\n"
        ),
    )
    .expect("write coproduct-arm fixture");
    let output = run_fixture(&root, &entry, "probe");
    let _ = std::fs::remove_dir_all(&root);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "a T slot judged with a coproduct and with one of its arms must be accepted, got exit {}:\n{stderr}",
        output.status,
    );
}

/// SHARED-T COPRODUCT CONTROL (side-chat review 83758 REQUEST_CHANGES item,
/// tidy-ant-251 2026-10-21): a Shape-typed argument CONSTRAINS T, and a Circle
/// LITERAL fills ANOTHER SLOT SHARING THAT SAME T. The previous coproduct
/// control used two independent identity<T> calls, so it never judged an arm
/// against a T already grounded to the coproduct parent. This is the shape that
/// must ACCEPT: the arm inhabits the parent T the call itself solved -- the
/// same relation refusal_cover_add relies on (a ByteRange arm snoc'd into a
/// FreeMonoid slot the call grounds to Extent).
#[test]
fn arm_at_second_slot_sharing_a_coproduct_constrained_type_variable_is_accepted() {
    let root = fixture_root("shared-t-coproduct");
    let entry = root.join("nominal_shared_t_coproduct.dag");
    std::fs::write(
        &entry,
        format!(
            "{LOCAL_PREAMBLE}\n\
             type Shape\n\
             \x20 = Circle {{ r: Int }}\n\
             \x20 | Square {{ side: Int }}\n\
             \n\
             fn keep_and_check<T>(keeper: T, candidate: T) -> T {{\n\
             \x20 candidate\n\
             }}\n\
             \n\
             fn probe() -> ProcessExit {{\n\
             \x20 let s: Shape = Square {{ side: 4 }}\n\
             \x20 let kept = keep_and_check(keeper: s, candidate: Circle {{ r: 3 }})\n\
             \x20 match kept {{\n\
             \x20\x20 Circle {{ r: r }} => if r == 3 {{ ExitSuccess }} else {{ ExitFailure {{ code: 1, reason: \"control shape broken\" }} }}\n\
             \x20\x20 Square {{ side: _ }} => ExitFailure {{ code: 1, reason: \"control shape broken\" }}\n\
             \x20 }}\n\
             }}\n"
        ),
    )
    .expect("write shared-t coproduct fixture");
    let output = run_fixture(&root, &entry, "probe");
    let _ = std::fs::remove_dir_all(&root);
    let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
    assert!(
        output.status.success(),
        "a Circle arm at a second slot whose T the call already solved to Shape must be accepted, got exit {}:\n{stderr}",
        output.status,
    );
}

fn run_fixture(root: &Path, entry: &Path, function: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_gunbc"))
        .current_dir(repo_root())
        .args([
            "run",
            "--source-root",
            root.to_string_lossy().as_ref(),
            "--source-root",
            "dag",
            "--source-root",
            "src/v2",
            "--entry",
            entry.to_string_lossy().as_ref(),
            "--function",
            function,
            "--claim-run",
        ])
        .output()
        .expect("invoke production gunbc on the nominal-admission fixture")
}
