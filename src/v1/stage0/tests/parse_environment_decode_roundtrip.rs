//! THE ENVIRONMENT LOADER, EXERCISED SO THAT AN IGNORED REVISION ARGUMENT CANNOT PASS.
//!
//! Two separate claims, because an earlier version of this file conflated them and could not have
//! caught the defect it was written to rule out.
//!
//! 1. DECODE. `evaluate_environment_in` over the live tree must reproduce `dag_parse_environment()`
//!    -- the environment compiled into this binary from the same declarations. An independent
//!    referent: the two agree only if evaluation, wire encoding and deserialization all preserved
//!    the environment.
//!
//! 2. REVISION SELECTION. `load_parse_environment_at` must read the bytes GIT holds, not the
//!    worktree's. The previous test called `evaluate_environment_in(workspace_root(), "HEAD")` and
//!    would have stayed green with the revision reader deleted outright, because the revision never
//!    selected anything. The discriminating case here is a revision whose environment DIFFERS from
//!    the worktree's, which is only authorable against a scratch repository -- so that is what the
//!    second test builds.

use std::path::Path;
use std::process::Command;

use v1_compiler::cli_run::namespace_baseline::{
    blob_id_at, decode_environment_wire, environment_load_refusal_text, evaluate_environment_in,
    kernel_set_serves_both, load_parse_environment_at, materialize_revision_paths,
    EnvironmentLoadRefusal, OperatorRowAuthority,
};
use v1_compiler::cli_run::workspace_root;
use v1_compiler::extdeps_languages_dag_syntax::dag_parse_environment;

const ENVIRONMENT_MODULE_PATH: &str = "dag/extdeps/languages/dag/syntax.dag";

/// The environment decoded from live-tree source equals the one compiled into this binary.
///
/// Whole-value equality rather than a field probe: every item form, every operator row with its
/// binding powers, every keyword literal and both keyword rosters must survive, because the loader
/// hands the WHOLE environment to the tokenizer and parser.
#[test]
fn the_decoder_reproduces_the_compiled_in_environment() {
    let root = workspace_root();
    let decoded = match evaluate_environment_in(&root, "live-tree") {
        Ok(env) => env,
        Err(e) => panic!(
            "decode refused on the live tree: {}",
            environment_load_refusal_text(&e)
        ),
    };
    assert_eq!(
        *decoded,
        *dag_parse_environment(),
        "environment decoded from source diverged from the compiled-in one"
    );
}

/// The oracle is not vacuous.
///
/// Without this, a decode producing an empty environment would pass the equality test on a day the
/// oracle was also empty. It pins the control itself -- the half a positive control usually leaves
/// unstated.
#[test]
fn the_compiled_in_environment_is_not_empty() {
    let compiled = dag_parse_environment();
    assert!(
        !compiled.syntax_spec.item_forms.is_empty(),
        "oracle has no item forms, so equality against it would prove nothing"
    );
    assert!(
        !compiled.syntax_spec.operators.is_empty(),
        "oracle has no operators, so equality against it would prove nothing"
    );
    assert!(
        !compiled.non_name_keywords.is_empty(),
        "oracle has no non-name keywords, so equality against it would prove nothing"
    );
}

fn git(dir: &Path, args: &[&str]) -> String {
    let out = Command::new("git")
        .args(args)
        .current_dir(dir)
        .output()
        .unwrap_or_else(|e| panic!("git {args:?} failed to start: {e}"));
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}

/// THE DISCRIMINATING CASE: a committed revision whose grammar differs from the worktree's.
///
/// A scratch repository is built holding the live tree's environment closure, committed, and then
/// the worktree's copy is EDITED to rename one keyword without committing. The loader is then asked
/// for the committed revision.
///
/// What this rules out, and nothing else does: a loader that reads the worktree would return the
/// EDITED environment and fail this test, and a loader that ignored its revision argument entirely
/// would do the same. The assertion is therefore about provenance, not about shape.
#[test]
fn the_loader_reads_the_revision_not_the_worktree() {
    let root = workspace_root();
    let scratch =
        std::env::temp_dir().join(format!("gunbc-env-revision-probe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch repo");

    // The whole `dag/` tree, so the closure resolves through the repository's own index -- acquired
    // through the loader's own route rather than a hand-shell pipeline beside it.
    materialize_revision_paths(&root, "HEAD", &scratch, &["dag"]).unwrap_or_else(|e| {
        panic!(
            "materializing the scratch corpus failed: {}",
            environment_load_refusal_text(&e)
        )
    });

    git(&scratch, &["init", "--quiet"]);
    git(&scratch, &["config", "user.email", "probe@example.invalid"]);
    git(&scratch, &["config", "user.name", "probe"]);
    git(&scratch, &["add", "-A"]);
    git(
        &scratch,
        &["commit", "--quiet", "-m", "baseline environment"],
    );
    let baseline = git(&scratch, &["rev-parse", "HEAD"]);

    // Now diverge the WORKTREE only: rename the `fn` item form's keyword. Uncommitted, so the
    // committed revision still carries the original spelling.
    let env_path = scratch.join(ENVIRONMENT_MODULE_PATH);
    let original = std::fs::read_to_string(&env_path).expect("read scratch environment module");
    let edited = original.replacen("keyword: \"fn\"", "keyword: \"zzfnzz\"", 1);
    assert_ne!(
        original, edited,
        "the probe could not find the `fn` keyword row to diverge, so it would prove nothing"
    );
    std::fs::write(&env_path, &edited).expect("write diverged worktree copy");

    // The committed blob exists -- the premise of the whole probe. The repository is named
    // explicitly rather than reached through the process cwd: `workspace_root()` memoizes the cwd
    // on first use and the other tests in this binary run concurrently, so a chdir here would
    // have made their colour depend on thread interleaving.
    let committed_id = blob_id_at(&scratch, &baseline, ENVIRONMENT_MODULE_PATH);
    assert!(
        matches!(committed_id, Ok(Some(_))),
        "the scratch revision has no environment blob, so the probe is not set up"
    );

    let loaded = match load_parse_environment_at(&scratch, &baseline) {
        Ok(env) => env,
        Err(e) => panic!(
            "loader refused on the scratch revision: {}",
            environment_load_refusal_text(&e)
        ),
    };

    let keywords: Vec<String> = loaded
        .syntax_spec
        .item_forms
        .iter()
        .map(|f| f.keyword.clone())
        .collect();
    assert!(
        keywords.iter().any(|k| k == "fn"),
        "the loader did not return the COMMITTED environment: `fn` is absent, so it read the \
         diverged worktree instead of revision {baseline}. keywords: {keywords:?}"
    );
    assert!(
        !keywords.iter().any(|k| k == "zzfnzz"),
        "the loader returned the WORKTREE environment: the uncommitted `zzfnzz` spelling reached \
         it, so the revision argument selected nothing. keywords: {keywords:?}"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}

const KERNEL_TYPES_PATH: &str = "dag/std/types.dag";

/// THE KERNEL GUARD DECIDES AT THE GRAIN OF THE NAME SET, NOT THE FILE'S BYTES.
///
/// Three revisions of one scratch repository, each compared against the live-tree baseline:
///
/// - a function-body and annotation edit to `dag/std/types.dag` changes the blob but cannot change
///   the kernel names, so the guard must answer `true`. The earlier whole-blob guard answered `false`
///   here and refused every such change repo-wide -- this is the discriminating RED for that shape;
/// - a revision that adds a kernel name must answer `false`: the single head map cannot speak for it;
/// - a revision whose `types.dag` does not declare `kernel_type_set` must REFUSE rather than
///   substitute this binary's set.
#[test]
fn the_kernel_guard_compares_names_not_bytes() {
    let root = workspace_root();
    let scratch =
        std::env::temp_dir().join(format!("gunbc-kernel-set-probe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch repo");
    materialize_revision_paths(&root, "HEAD", &scratch, &["dag"]).unwrap_or_else(|e| {
        panic!(
            "materializing the scratch corpus failed: {}",
            environment_load_refusal_text(&e)
        )
    });
    git(&scratch, &["init", "--quiet"]);
    git(&scratch, &["config", "user.email", "probe@example.invalid"]);
    git(&scratch, &["config", "user.name", "probe"]);
    git(&scratch, &["add", "-A"]);
    git(&scratch, &["commit", "--quiet", "-m", "baseline"]);
    let baseline = git(&scratch, &["rev-parse", "HEAD"]);

    let types_path = scratch.join(KERNEL_TYPES_PATH);
    let original = std::fs::read_to_string(&types_path).expect("read scratch types.dag");
    let commit_variant = |label: &str, text: &str| -> String {
        std::fs::write(&types_path, text).expect("write types.dag variant");
        git(&scratch, &["commit", "--quiet", "-am", label]);
        let id = git(&scratch, &["rev-parse", "HEAD"]);
        git(
            &scratch,
            &["checkout", "--quiet", &baseline, "--", KERNEL_TYPES_PATH],
        );
        git(
            &scratch,
            &["commit", "--quiet", "--allow-empty", "-am", "restore"],
        );
        id
    };

    // Body-only: the kernel set is untouched, the bytes are not.
    let body_edit = original.replacen("    Absent => false\n", "    Absent => (1 == 2)\n", 1);
    assert_ne!(
        original, body_edit,
        "the probe could not find is_kernel_type's body to edit"
    );
    let body_rev = commit_variant("body-only edit", &body_edit);
    assert_ne!(
        blob_id_at(&scratch, &baseline, KERNEL_TYPES_PATH).ok(),
        blob_id_at(&scratch, &body_rev, KERNEL_TYPES_PATH).ok(),
        "the body edit did not change the blob, so the probe does not exercise the name-set arm"
    );
    match kernel_set_serves_both(&scratch, &body_rev, &baseline) {
        Ok(true) => {}
        other => panic!(
            "a function-body edit to {KERNEL_TYPES_PATH} was judged to change the kernel set: {:?}",
            other.map_err(|e| environment_load_refusal_text(&e))
        ),
    }

    // A kernel name added: the base's set is not this binary's.
    let widened = original.replacen("\"Bytes\": true", "\"Bytes\": true, \"ZzProbe\": true", 1);
    assert_ne!(
        original, widened,
        "the probe could not find the kernel set to widen"
    );
    let widened_rev = commit_variant("kernel name added", &widened);
    match kernel_set_serves_both(&scratch, &widened_rev, &baseline) {
        Ok(false) => {}
        other => panic!(
            "a base revision declaring an extra kernel name was judged served by this binary's set: {:?}",
            other.map_err(|e| environment_load_refusal_text(&e))
        ),
    }

    // The set is not declared at all: refuse as an unreadable KERNEL SET, never substitute.
    let renamed = original.replacen("data kernel_type_set:", "data zz_kernel_type_set:", 1);
    assert_ne!(
        original, renamed,
        "the probe could not find the kernel set declaration"
    );
    let unreadable_rev = commit_variant(
        "kernel set undeclared",
        &renamed.replace("map_get(kernel_type_set,", "map_get(zz_kernel_type_set,"),
    );
    match kernel_set_serves_both(&scratch, &unreadable_rev, &baseline) {
        Err(EnvironmentLoadRefusal::KernelSetNotReadable { revision, cause }) => {
            assert_eq!(
                revision, unreadable_rev,
                "the refusal named a different revision than the one whose set was unreadable"
            );
            assert!(
                cause.contains("`kernel_type_set` is not declared by"),
                "the refusal did not locate the missing declaration: {cause}"
            );
        }
        other => panic!(
            "a base revision with no kernel_type_set declaration was not refused as an unreadable \
             kernel set: {other:?}"
        ),
    }

    // The HEAD has no declaring file: the compiled-in set must not stand in for an absent authority.
    git(&scratch, &["rm", "--quiet", KERNEL_TYPES_PATH]);
    git(&scratch, &["commit", "--quiet", "-m", "types.dag removed"]);
    let headless_rev = git(&scratch, &["rev-parse", "HEAD"]);
    match kernel_set_serves_both(&scratch, &baseline, &headless_rev) {
        Err(EnvironmentLoadRefusal::KernelSetNotReadable { revision, .. }) => assert_eq!(
            revision, headless_rev,
            "the absent-head refusal named the wrong revision"
        ),
        other => panic!(
            "a head revision with no {KERNEL_TYPES_PATH} was answered from the compiled-in set: {other:?}"
        ),
    }
    // ...and an absent file on BOTH sides is not equality.
    match kernel_set_serves_both(&scratch, &headless_rev, &headless_rev) {
        Err(EnvironmentLoadRefusal::KernelSetNotReadable { .. }) => {}
        other => panic!("two absent declaring files were judged to agree: {other:?}"),
    }

    let _ = std::fs::remove_dir_all(&scratch);
}

/// THE HISTORICAL-REVISION CASE (the prefix/postfix-operator declaration's migration).
///
/// A revision older than the commit that declared `prefix_operators` / `postfix_operators` in
/// `std.syntax` encodes a SyntaxSpec with NEITHER field — the type did not have them, and the
/// parser read `min_bp: 12` at the prefix arms and literals at the postfix arms instead. This test
/// builds exactly such a revision in a scratch repository — the materialized environment module and
/// its `std.syntax` closure file, edited back to their pre-declaration shape, committed — and loads
/// it through `load_parse_environment_at`, the base/head loader's own route. Before the migration
/// in `decode_environment_wire` this refused with a serde "missing field `prefix_operators`"-shaped
/// `ValueNotDecodable`, which the namespace baseline read as `NotEvaluated`; it must now decode,
/// and the filled rows must be the HISTORICAL powers (prefix 12/12, postfix 21/21/13) that make
/// this binary's table-driven parser reproduce that revision's own parse decisions — not the current
/// 18/19/21, and the rest of the environment must come through unchanged.
#[test]
fn the_loader_decodes_a_pre_declaration_revision_with_historical_rows() {
    let root = workspace_root();
    let scratch =
        std::env::temp_dir().join(format!("gunbc-env-historical-probe-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&scratch);
    std::fs::create_dir_all(&scratch).expect("create scratch repo");

    // The environment closure AND std.syntax, both edited back below. std.syntax is not in the
    // environment module's import closure unless imported by name; materializing the environment
    // module's closure brings it, so materialize the closure through the same path the loader uses
    // and edit whatever landed.
    materialize_revision_paths(&root, "HEAD", &scratch, &["dag"]).unwrap_or_else(|e| {
        panic!(
            "materializing the scratch corpus failed: {}",
            environment_load_refusal_text(&e)
        )
    });

    let env_path = scratch.join(ENVIRONMENT_MODULE_PATH);
    let std_syntax_path = scratch.join("dag/std/syntax.dag");
    let env_original = std::fs::read_to_string(&env_path).expect("read scratch environment module");
    let std_original = std::fs::read_to_string(&std_syntax_path).expect("read scratch std.syntax");

    // Edit the type back: drop the two declared-row fields from SyntaxSpec...
    let std_edited = std_original
        .replace("  prefix_operators: List<PrefixOperatorSpec>\n", "")
        .replace("  postfix_operators: List<PostfixOperatorSpec>\n", "");
    // Whichever field the checked-out revision already lacks stays lacking; nothing to strip.
    let stripped_std = std_edited != std_original;
    // ...and the environment module back: drop the two data declarations and the two constructor
    // fields. (These are exactly the lines the declaration commit added; deleting them reproduces
    // the historical source.)
    let mut env_edited = env_original.clone();
    let mut stripped_env = false;
    for (field, data_marker) in [
        ("prefix", "data dag_prefix_operators:"),
        ("postfix", "data dag_postfix_operators:"),
    ] {
        let ctor_line = format!("  {field}_operators: dag_{field}_operators,\n");
        if env_edited.contains(&ctor_line) {
            env_edited = env_edited.replace(&ctor_line, "");
            stripped_env = true;
        }
        if env_edited.contains(data_marker) {
            env_edited = drop_data_block(&env_edited, data_marker);
            stripped_env = true;
        }
    }
    assert!(
        stripped_std || stripped_env,
        "the revision carried neither row field to strip; the test's premise (a revision that          predates the declaration) cannot be built from it"
    );

    std::fs::write(&std_syntax_path, &std_edited).expect("write historical std.syntax");
    std::fs::write(&env_path, &env_edited).expect("write historical environment");

    git(&scratch, &["init", "--quiet"]);
    git(&scratch, &["config", "user.email", "probe@example.invalid"]);
    git(&scratch, &["config", "user.name", "probe"]);
    git(&scratch, &["add", "-A"]);
    git(
        &scratch,
        &["commit", "--quiet", "-m", "pre-declaration environment"],
    );
    let historical_rev = git(&scratch, &["rev-parse", "HEAD"]);

    let loaded = match load_parse_environment_at(&scratch, &historical_rev) {
        Ok(env) => env,
        Err(e) => panic!(
            "the loader refused a pre-declaration revision instead of migrating it: {}",
            environment_load_refusal_text(&e)
        ),
    };

    let prefix: Vec<(String, i64)> = loaded
        .syntax_spec
        .prefix_operators
        .iter()
        .map(|r| (r.symbol.clone(), r.binding_power))
        .collect();
    let postfix: Vec<(String, i64)> = loaded
        .syntax_spec
        .postfix_operators
        .iter()
        .map(|r| (r.symbol.clone(), r.binding_power))
        .collect();
    assert_eq!(
        prefix,
        vec![("-".to_string(), 12i64), ("!".to_string(), 12i64)],
        "a historical environment did not receive the parser's historical prefix powers"
    );
    assert_eq!(
        postfix,
        vec![
            ("(".to_string(), 21i64),
            ("[".to_string(), 21i64),
            ("as".to_string(), 13i64),
            // The record-literal arm's historical gate: the parser refused the brace when
            // `14 <= min_bp`, i.e. consumed at min_bp <= 13 -- consume-iff-scale 13.
            ("{".to_string(), 13i64)
        ],
        "a historical environment did not receive the parser's historical postfix gates"
    );

    // The migration fills ONLY the absent rows: the rows the revision DID declare — its operator
    // table and item forms — come through as that revision wrote them, which here means equal to
    // the environment compiled into this binary (neither was touched by the declaration commit).
    let live = dag_parse_environment();
    assert_eq!(
        loaded.syntax_spec.operators, live.syntax_spec.operators,
        "the migration disturbed the operator rows the historical revision already declared"
    );
    assert_eq!(
        loaded.syntax_spec.item_forms, live.syntax_spec.item_forms,
        "the migration disturbed the item forms the historical revision already declared"
    );

    let _ = std::fs::remove_dir_all(&scratch);
}

/// Drop a top-level `data NAME ... = [...]` declaration, bracket-balanced, with its leading comment
/// block kept out of scope: only the declaration itself is removed.
fn drop_data_block(source: &str, marker: &str) -> String {
    let start = match source.find(marker) {
        Some(i) => i,
        None => panic!("{marker} was not present to strip"),
    };
    let open = source[start..]
        .find('[')
        .unwrap_or_else(|| panic!("{marker} carried no list initializer"));
    let mut depth = 0usize;
    let bytes = source.as_bytes();
    let mut end = start + open;
    let mut closed = false;
    // Balance STRUCTURAL brackets only: the row lists in this file include `symbol: "["` - a
    // bracket inside a string literal - which a naive count carries past zero and reports as an
    // unbalanced initializer. Skip string bodies (with backslash escapes) and line comments.
    let mut in_string = false;
    let mut escaped = false;
    while end < bytes.len() {
        let byte = bytes[end];
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
        } else {
            match byte {
                b'"' => in_string = true,
                b'/' if bytes.get(end + 1) == Some(&b'/') => {
                    while end < bytes.len() && bytes[end] != b'\n' {
                        end += 1;
                    }
                    continue;
                }
                b'[' => depth += 1,
                b']' => {
                    depth -= 1;
                    if depth == 0 {
                        closed = true;
                        break;
                    }
                }
                _ => {}
            }
        }
        end += 1;
    }
    // Never slice past the end: an unbalanced scan is a refusal naming the marker, not a panic
    // with a byte-index message.
    assert!(closed, "{marker} carries an unbalanced list initializer");
    let mut end = end + 1;
    while end < bytes.len() && bytes[end] != b'\n' {
        end += 1;
    }
    if end < bytes.len() {
        end += 1;
    }
    // Snap to a char boundary: the line we skipped may hold multi-byte characters (comments in this
    // repository do), and the caller slices by these byte offsets.
    while end < bytes.len() && (bytes[end] & 0xC0) == 0x80 {
        end += 1;
    }
    // Byte-indexed slicing of a str panics at a multi-byte boundary; go through bytes so the strip
    // is byte-safe regardless of what the declaration's comments contain.
    let bytes = source.as_bytes();
    let mut out = String::with_capacity(source.len());
    out.push_str(std::str::from_utf8(&bytes[..start]).expect("strip start is char-aligned"));
    out.push_str(std::str::from_utf8(&bytes[end..]).expect("strip end is char-aligned"));
    out
}

/// THE WIRE-LEVEL CASE, exercised directly at the decode seam.
///
/// The current encoder's output for the live environment, with the two row fields removed from the
/// SyntaxSpec object, must decode (through `decode_environment_wire`) to the historical rows; and —
/// the discrimination — a document that HAS the fields is never touched, so a current-encoder bug
/// that dropped them would not be masked here: the live roundtrip above compares whole values
/// against the declared 18/19/21, and a current environment whose rows went missing on the way out
/// decodes to 12/13/21 there and fails THAT equality.
#[test]
fn the_wire_migration_fills_only_absent_row_fields() {
    let live = dag_parse_environment();
    let wire = serde_json::to_value(&*live.syntax_spec).expect("encode the live SyntaxSpec");

    // Present: untouched. Decoding the live object must give the DECLARED powers back.
    let declared = decode_environment_wire(
        &serde_json::json!({
            "syntax_spec": wire,
            "non_name_keywords": {},
            "non_pattern_binder_keywords": {},
        }),
        "test-present",
        OperatorRowAuthority::Declared,
    )
    .expect("a current-shaped environment must decode");
    assert_eq!(
        declared
            .syntax_spec
            .prefix_operators
            .iter()
            .map(|r| r.binding_power)
            .collect::<Vec<_>>(),
        vec![18i64, 18i64],
        "the migration changed a document that declared its own prefix rows"
    );
    assert_eq!(
        declared
            .syntax_spec
            .postfix_operators
            .iter()
            .map(|r| r.binding_power)
            .collect::<Vec<_>>(),
        vec![21i64, 21i64, 19i64, 18i64],
        "the migration changed a document that declared its own postfix rows"
    );

    // Absent: filled with the historical powers.
    let mut historical = wire.clone();
    let obj = historical
        .as_object_mut()
        .expect("SyntaxSpec encodes as an object");
    obj.remove("prefix_operators");
    obj.remove("postfix_operators");
    let filled = decode_environment_wire(
        &serde_json::json!({
            "syntax_spec": historical,
            "non_name_keywords": {},
            "non_pattern_binder_keywords": {},
        }),
        "test-absent",
        OperatorRowAuthority::PredatesDeclaration,
    )
    .expect("a pre-declaration-shaped environment must decode through the migration");
    assert_eq!(
        filled
            .syntax_spec
            .prefix_operators
            .iter()
            .map(|r| r.binding_power)
            .collect::<Vec<_>>(),
        vec![12i64, 12i64],
        "a pre-declaration document did not receive the historical prefix powers"
    );
    assert_eq!(
        filled
            .syntax_spec
            .postfix_operators
            .iter()
            .map(|r| r.binding_power)
            .collect::<Vec<_>>(),
        vec![21i64, 21i64, 13i64, 13i64],
        "a pre-declaration document did not receive the historical postfix gates"
    );
    // Everything the document DID carry survives.
    assert_eq!(filled.syntax_spec.operators, live.syntax_spec.operators);

    // THE ESCAPE-HATCH REFUSAL: the SAME field-less document, decoded under the authority a
    // CURRENT revision gets, must REFUSE. A shape-detected fill would have restored powers 12/13/21
    // here and made `-1 - 2` mean `-(1 - 2)` again -- the silent-wrongness this migration must never
    // do. The authority is decided from the revision's own source (see operator_row_authority), so
    // nothing in the bytes can earn a fill.
    let refused = decode_environment_wire(
        &serde_json::json!({
            "syntax_spec": historical,
            "non_name_keywords": {},
            "non_pattern_binder_keywords": {},
        }),
        "test-current-revision-encoding-defect",
        OperatorRowAuthority::Declared,
    );
    match refused {
        Err(EnvironmentLoadRefusal::ValueNotDecodable { revision, cause }) => {
            assert_eq!(revision, "test-current-revision-encoding-defect");
            assert!(
                cause.contains("prefix_operators") || cause.contains("postfix_operators"),
                "the refusal must name the missing field, not a guess: {cause}"
            );
        }
        Err(other) => panic!(
            "a declared revision with a field-less wire must refuse with ValueNotDecodable, got {}",
            environment_load_refusal_text(&other)
        ),
        Ok(env) => panic!(
            "a declared revision's environment silently gained historical powers: prefix {:?} postfix {:?}",
            env.syntax_spec.prefix_operators.iter().map(|r| r.binding_power).collect::<Vec<_>>(),
            env.syntax_spec.postfix_operators.iter().map(|r| r.binding_power).collect::<Vec<_>>(),
        ),
    }
}
