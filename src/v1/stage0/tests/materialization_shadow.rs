//! Production route for the pre-registered fresh-process experiment in
//! docs/measurements/v1_compiler_materialization. No stored answer is served.
use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;

const QUESTION: &str = "v1.std.core::build_newline_index";
const PREFIX: &str = "[materialization-shadow] ";
const STORE_ENV: &str = "GUNBC_MATERIALIZATION_SHADOW_DIR";
const ENTRY: &str = r#"module shadow.entry
import shadow.helper { answer }
type ProcessExit = ExitSuccess | ExitFailure { code: Int, reason: String }
fn main() -> ProcessExit {
  if answer() == 1 { ExitSuccess } else { ExitFailure { code: 7, reason: "observed changed helper" } }
}
"#;

#[derive(Debug, Deserialize)]
struct Event {
    question: String,
    logical_source: String,
    input_digest: String,
    output_digest: String,
    lookup: String,
    comparison: String,
}

struct Reading {
    code: i32,
    stdout: Vec<u8>,
    stderr: String,
    events: Vec<Event>,
}

struct Experiment {
    repo: PathBuf,
    dir: PathBuf,
}

fn create_experiment() -> Experiment {
    let repo = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../..")
        .canonicalize()
        .unwrap();
    let target = repo.join("target");
    fs::create_dir_all(&target).unwrap();
    let dir = target.join(format!(
        "materialization-shadow-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    fs::create_dir(&dir).unwrap();
    fs::create_dir(dir.join("src")).unwrap();
    fs::write(dir.join("src/entry.dag"), ENTRY).unwrap();
    fs::write(
        dir.join("src/unrelated.dag"),
        "module shadow.unrelated\nfn nobody_calls() -> Bool { true }\n",
    )
    .unwrap();
    let case = Experiment { repo, dir };
    write_helper(&case, 1);
    case
}

fn write_helper(case: &Experiment, literal: i32) {
    fs::write(
        case.dir.join("src/helper.dag"),
        format!("module shadow.helper\n// é\nfn answer() -> Int {{ {literal} }}\n"),
    )
    .unwrap();
}

fn run(case: &Experiment, store: Option<&Path>, expected_code: i32) -> Reading {
    let src = case.dir.join("src");
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_gunbc"));
    cmd.current_dir(&case.repo)
        .env_remove(STORE_ENV)
        .arg("run")
        .arg("--source-root")
        .arg(&src)
        .arg("--entry")
        .arg(src.join("entry.dag"))
        .args(["--function", "main"]);
    if let Some(store) = store {
        cmd.env(STORE_ENV, store);
    }
    let output = cmd.output().expect("execute gunbc run");
    let stderr = String::from_utf8(output.stderr).unwrap();
    let code = output.status.code().expect("gunbc exited by signal");
    assert_eq!(code, expected_code, "{stderr}");
    let events = stderr
        .lines()
        .filter_map(|l| l.strip_prefix(PREFIX))
        .map(|s| serde_json::from_str(s).expect("malformed shadow event"))
        .collect();
    Reading {
        code,
        stderr,
        stdout: output.stdout,
        events,
    }
}

fn sample<'a>(
    case: &Experiment,
    reading: &'a Reading,
    file: &str,
    lookup: &str,
    comparison: &str,
) -> &'a Event {
    let source = case.dir.join("src").join(file);
    let mut events = reading.events.iter().filter(|event| {
        event.question == QUESTION && case.repo.join(&event.logical_source) == source
    });
    let first = events.next().unwrap_or_else(|| {
        panic!(
            "no {QUESTION} event for {}: {}",
            source.display(),
            reading.stderr
        )
    });
    assert_eq!(
        (&*first.lookup, &*first.comparison),
        (lookup, comparison),
        "{first:?}"
    );
    for d in [&first.input_digest, &first.output_digest] {
        assert!(
            d.len() == 64
                && d.bytes()
                    .all(|c| c.is_ascii_digit() || (b'a'..=b'f').contains(&c)),
            "{first:?}"
        );
    }
    for event in events {
        assert_eq!(
            (&event.input_digest, &event.output_digest),
            (&first.input_digest, &first.output_digest)
        );
        let subsequent = if lookup == "corrupt" {
            ("corrupt", "unavailable")
        } else {
            ("hit", "equal")
        };
        assert_eq!(
            (&*event.lookup, &*event.comparison),
            subsequent,
            "{event:?}"
        );
    }
    first
}

#[test]
fn fresh_process_predictions_match_observed_compiler_results() {
    let case = create_experiment();
    let store = case.dir.join("store");
    let baseline = run(&case, Some(&store), 0);
    let entry = sample(&case, &baseline, "entry.dag", "miss", "absent");
    let helper = sample(&case, &baseline, "helper.dag", "miss", "absent");

    let same = |run: &Reading, state, relation| {
        for (file, original) in [("entry.dag", entry), ("helper.dag", helper)] {
            let seen = sample(&case, run, file, state, relation);
            assert_eq!(
                (&seen.input_digest, &seen.output_digest),
                (&original.input_digest, &original.output_digest)
            );
        }
        assert_eq!(run.stdout, baseline.stdout);
    };
    same(&run(&case, Some(&store), 0), "hit", "equal");
    let unrelated = case.dir.join("src/unrelated.dag");
    fs::write(
        &unrelated,
        "module shadow.unrelated\nfn nobody_calls() -> Bool { false }\n",
    )
    .unwrap();
    let unrelated_run = run(&case, Some(&store), 0);
    same(&unrelated_run, "hit", "equal");
    assert!(!unrelated_run
        .events
        .iter()
        .any(|e| e.question == QUESTION && case.repo.join(&e.logical_source) == unrelated));

    write_helper(&case, 2);
    let edited = run(&case, Some(&store), 7);
    assert_eq!(
        sample(&case, &edited, "entry.dag", "hit", "equal").output_digest,
        entry.output_digest
    );
    let changed = sample(&case, &edited, "helper.dag", "miss", "absent");
    assert_ne!(changed.input_digest, helper.input_digest);
    assert_ne!(changed.output_digest, helper.output_digest);
    let uncached = run(&case, None, 7);
    assert!(uncached.events.is_empty());
    assert_eq!(
        (uncached.code, uncached.stdout),
        (edited.code, edited.stdout)
    );
    assert!(
        uncached.stderr.contains("observed changed helper")
            && edited.stderr.contains("observed changed helper")
    );

    write_helper(&case, 1);
    same(&run(&case, Some(&store), 0), "hit", "equal");
    same(
        &run(&case, Some(&case.dir.join("empty-store")), 0),
        "miss",
        "absent",
    );

    let record = store.join("newline-index-v1").join(&helper.input_digest);
    let mut bytes = fs::read(&record).unwrap();
    bytes[0] ^= 1;
    fs::write(record, bytes).unwrap();
    let corrupt = run(&case, Some(&store), 0);
    sample(&case, &corrupt, "entry.dag", "hit", "equal");
    assert_eq!(
        sample(&case, &corrupt, "helper.dag", "corrupt", "unavailable").output_digest,
        helper.output_digest
    );
    assert_eq!(corrupt.stdout, baseline.stdout);
    fs::remove_dir_all(&case.dir).unwrap();
}

/// Independent of the shadow observer: compare fresh, typed compiler answers over a controlled
/// edit. These observations make no claim about a cache lookup, payload size or elapsed time.
#[test]
fn body_edits_change_some_compiler_answers_and_preserve_others() {
    use std::rc::Rc;
    use v1_compiler::v1_compiler_compile::{
        compile_to_resolved, dag_parse_environment, front_end_sources, SourceFile,
    };
    use v1_compiler::v1_compiler_normalize::normalize_graph;
    use v1_compiler::v1_compiler_tokenize::tokenize_artifact;
    use v1_compiler::v1_std_core::LeafOwner;

    let source = |name, literal| {
        Rc::new(SourceFile {
            path: format!("{name}.dag"),
            content: format!("module shadow.{name}\nfn answer() -> Int {{ {literal} }}\n"),
        })
    };
    let first = source("helper", 1);
    let edit = source("helper", 2);
    let lex = |s: &SourceFile| {
        tokenize_artifact(s.content.clone(), s.path.clone(), dag_parse_environment())
    };
    assert_ne!(lex(&first).tokens, lex(&edit).tokens);
    let prepare = |s: Rc<SourceFile>| front_end_sources(Rc::new(im::vector![s]));
    let original = prepare(first.clone());
    let edited = prepare(edit.clone());
    assert!(original.graph.is_some() && edited.graph.is_some());
    assert_eq!(
        original.newline_indices[0].offsets,
        edited.newline_indices[0].offsets
    );
    assert_ne!(
        original.newline_indices[0].char_codes,
        edited.newline_indices[0].char_codes
    );
    assert_eq!(original.intern_table.strings, edited.intern_table.strings);
    assert_eq!(original.intern_table.index, edited.intern_table.index);
    assert_eq!(
        original.intern_table.authored_token_ordinals,
        edited.intern_table.authored_token_ordinals
    );
    for front in [original, edited] {
        let indices = front
            .newline_indices
            .iter()
            .map(|index| (index.file.clone(), index.clone()))
            .collect();
        let normalized = normalize_graph(front.graph.clone().unwrap(), Rc::new(indices));
        assert!(
            normalized.diagnostics.is_empty(),
            "{:?}",
            normalized.diagnostics
        );
    }
    let leaf = |sources| {
        let compiled = compile_to_resolved(Rc::new(sources));
        let graph = compiled.graph.as_ref().expect("missing resolved graph");
        graph
            .item_leaf_owner_modules
            .get("answer")
            .expect("missing answer declaration")
            .clone()
    };
    assert!(
        matches!(leaf(im::vector![first.clone()]).as_ref(), LeafOwner::SingleOwner { module } if module == "shadow.helper")
    );
    assert_eq!(leaf(im::vector![first.clone()]), leaf(im::vector![edit]));
    assert!(matches!(
        leaf(im::vector![first, source("other", 1)]).as_ref(),
        LeafOwner::LeafAmbiguous
    ));
}
