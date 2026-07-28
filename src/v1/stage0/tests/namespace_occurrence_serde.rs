use im::HashMap;
use std::rc::Rc;
use std::time::{Duration, Instant};

use v1_compiler::std_occurrence_identity::{
    occurrence_id_allocator_initial, DeclarationOccurrence, OccurrenceCategory,
    OccurrenceContainmentPath, OccurrenceId, OccurrenceIdAllocator, OccurrenceIndex,
    OccurrenceIndexEntry, OccurrenceProjection, OccurrenceTransport, OccurrenceTransportRefusal,
    ReferenceOccurrence,
};
use v1_compiler::std_types::SourceSpan;
use v1_compiler::v1_compiler_compile::{
    compile_to_resolved, resolve_frontend_occurrence_transport, ResolvedPipelineResult, SourceFile,
};
use v1_compiler::v1_compiler_parse::{parse_with_table_in_occurrence_scope, ParseWithTableResult};
use v1_compiler::v1_compiler_tokenize::tokenize;
use v1_compiler::v1_std_core::{
    build_newline_index, diagnostic_to_span, empty_intern_table, CompilerDiagnostic, NewlineIndex,
};

fn parse_source(
    source: &str,
    file: &str,
    allocator: OccurrenceIdAllocator,
) -> Rc<ParseWithTableResult> {
    let mut source_indices: HashMap<String, Rc<NewlineIndex>> = HashMap::new();
    source_indices.insert(
        file.to_string(),
        build_newline_index(file.to_string(), source.to_string()),
    );
    parse_with_table_in_occurrence_scope(
        tokenize(source.to_string(), file.to_string()),
        Rc::new(source_indices),
        empty_intern_table(),
        allocator,
    )
}

fn repeated_module(functions: usize) -> String {
    let mut source = String::from("module occurrence.scale\n\n");
    for index in 0..functions {
        source.push_str(&format!("fn f_{index}(same: Int) -> Int {{ same }}\n"));
    }
    source
}

fn max_rss_kib() -> i64 {
    let mut usage = std::mem::MaybeUninit::<libc::rusage>::uninit();
    let rc = unsafe { libc::getrusage(libc::RUSAGE_SELF, usage.as_mut_ptr()) };
    assert_eq!(rc, 0, "getrusage must be available for the size receipt");
    unsafe { usage.assume_init().ru_maxrss }
}

fn occurrence_span(file: &str, start: i64, end: i64) -> Rc<SourceSpan> {
    Rc::new(SourceSpan {
        file: file.to_string(),
        start,
        end,
    })
}

fn occurrence_path(occurrence: OccurrenceId) -> Rc<OccurrenceContainmentPath> {
    Rc::new(OccurrenceContainmentPath {
        ancestors: Rc::new(vec![].into()),
        terminal: occurrence,
    })
}

fn occurrence_index_entry(
    occurrence: OccurrenceId,
    span: Rc<SourceSpan>,
) -> Rc<OccurrenceIndexEntry> {
    Rc::new(OccurrenceIndexEntry {
        projection: Rc::new(OccurrenceProjection {
            occurrence,
            authored_name: "same".to_string(),
            diagnostic_span: span,
        }),
        containment: occurrence_path(occurrence),
    })
}

fn occurrence_transport(
    entries: Vec<Rc<OccurrenceIndexEntry>>,
    declarations: Vec<Rc<DeclarationOccurrence>>,
    references: Vec<Rc<ReferenceOccurrence>>,
) -> Rc<OccurrenceTransport> {
    Rc::new(OccurrenceTransport {
        index: Rc::new(OccurrenceIndex {
            entries: Rc::new(entries.into()),
        }),
        declarations: Rc::new(declarations.into()),
        references: Rc::new(references.into()),
    })
}

#[test]
fn production_frontend_refuses_invalid_occurrence_transport() {
    let occurrence = OccurrenceId { value: 17 };
    let span = occurrence_span("occurrence_refusal.dag", 4, 9);
    let declaration = Rc::new(DeclarationOccurrence {
        occurrence,
        containment: occurrence_path(occurrence),
        category: OccurrenceCategory::LexicalValueOccurrence,
        diagnostic_span: span.clone(),
    });
    let reference = Rc::new(ReferenceOccurrence {
        occurrence,
        containment: occurrence_path(occurrence),
        category: OccurrenceCategory::CallableOccurrence,
        diagnostic_span: span.clone(),
    });
    let entry = occurrence_index_entry(occurrence, span.clone());

    let valid = resolve_frontend_occurrence_transport(
        Rc::new(vec![].into()),
        Rc::new(HashMap::new()),
        occurrence_transport(vec![], vec![], vec![]),
    );
    assert!(
        valid.graph.is_some() && valid.diagnostics.is_empty(),
        "valid empty transport must reach the existing resolver"
    );

    let cases = [
        (
            occurrence_transport(vec![], vec![declaration.clone()], vec![]),
            "missing",
        ),
        (
            occurrence_transport(
                vec![entry.clone(), entry.clone()],
                vec![declaration.clone()],
                vec![],
            ),
            "duplicate",
        ),
        (
            occurrence_transport(vec![entry], vec![declaration], vec![reference]),
            "wrong-category",
        ),
    ];

    for (transport, expected) in cases {
        let result = resolve_frontend_occurrence_transport(
            Rc::new(vec![].into()),
            Rc::new(HashMap::new()),
            transport,
        );
        assert!(
            result.graph.is_none(),
            "{expected} transport must stop before resolver acceptance"
        );
        assert_eq!(
            result.diagnostics.len(),
            1,
            "{expected} transport must produce one typed diagnostic"
        );
        let diagnostic = result.diagnostics[0].diagnostic.clone();
        assert_eq!(
            diagnostic_to_span(diagnostic.clone()),
            span,
            "{expected} refusal must preserve its authored location"
        );
        match (&*diagnostic, expected) {
            (CompilerDiagnostic::OccurrenceTransportViolation { refusal }, "missing") => {
                assert!(matches!(
                    &**refusal,
                    OccurrenceTransportRefusal::MissingAuthoredOccurrenceIdentity { .. }
                ))
            }
            (CompilerDiagnostic::OccurrenceTransportViolation { refusal }, "duplicate") => {
                assert!(matches!(
                    &**refusal,
                    OccurrenceTransportRefusal::DuplicateAuthoredOccurrenceIdentity { .. }
                ))
            }
            (CompilerDiagnostic::OccurrenceTransportViolation { refusal }, "wrong-category") => {
                assert!(matches!(
                    &**refusal,
                    OccurrenceTransportRefusal::WrongOccurrenceCategory { .. }
                ))
            }
            _ => panic!("{expected} transport returned the wrong typed diagnostic"),
        }
    }
}

#[test]
fn occurrence_transport_sidecar_round_trips_with_paths() {
    let source = "module occurrence.serde\n\nfn left(x: Int) -> Int { x }\nfn right(x: Int) -> Int { left(x) }\n";
    let parsed = parse_source(
        source,
        "occurrence_serde.dag",
        occurrence_id_allocator_initial(),
    );
    assert!(parsed.result.error.is_none(), "{:?}", parsed.result.error);
    assert!(!parsed.occurrence_transport.index.entries.is_empty());
    assert!(!parsed.occurrence_transport.declarations.is_empty());
    assert!(!parsed.occurrence_transport.references.is_empty());

    let encoded = serde_json::to_vec(&parsed).expect("serialize occurrence parse result");
    let decoded: Rc<ParseWithTableResult> =
        serde_json::from_slice(&encoded).expect("deserialize occurrence parse result");
    assert_eq!(decoded, parsed);

    let parsed_with_perturbed_span_file = parse_source(
        source,
        "different_span_file.dag",
        occurrence_id_allocator_initial(),
    );
    let original_ids: Vec<i64> = parsed
        .occurrence_transport
        .index
        .entries
        .iter()
        .map(|entry| entry.projection.occurrence.value)
        .collect();
    let perturbed_ids: Vec<i64> = parsed_with_perturbed_span_file
        .occurrence_transport
        .index
        .entries
        .iter()
        .map(|entry| entry.projection.occurrence.value)
        .collect();
    assert_eq!(
        original_ids, perturbed_ids,
        "filename/span evidence must not participate in occurrence identity"
    );

    let resolved = compile_to_resolved(Rc::new(im::vector![Rc::new(SourceFile {
        path: "occurrence_serde.dag".to_string(),
        content: source.to_string(),
    })]));
    let graph = resolved
        .graph
        .as_ref()
        .expect("production compile must preserve a resolved graph");
    let typed_module = graph
        .modules
        .get(0)
        .expect("production compile must preserve its typed module");
    let typed_transport = typed_module
        .occurrence_transport
        .as_ref()
        .expect("typed module must preserve its parser-owned occurrence sidecar");
    assert_eq!(
        typed_transport, &parsed.occurrence_transport,
        "inference/rebuild must preserve the exact parser-owned sidecar"
    );

    let resolved_bytes = serde_json::to_vec(&resolved)
        .expect("serialize resolved graph with typed-module occurrence sidecar");
    let resolved_decoded: Rc<ResolvedPipelineResult> = serde_json::from_slice(&resolved_bytes)
        .expect("deserialize resolved graph with typed-module occurrence sidecar");
    assert_eq!(
        resolved_decoded, resolved,
        "resolved-graph serde must preserve typed-module occurrence identity"
    );
}

#[test]
fn occurrence_transport_serialization_and_parse_cost_scale_linearly() {
    let small_source = repeated_module(32);
    let small_start = Instant::now();
    let small = parse_source(
        &small_source,
        "occurrence_scale_small.dag",
        occurrence_id_allocator_initial(),
    );
    let small_elapsed = small_start.elapsed();
    let small_bytes = serde_json::to_vec(&small.occurrence_transport)
        .expect("serialize small occurrence transport")
        .len();
    let small_rss = max_rss_kib();

    let large_source = repeated_module(64);
    let large_start = Instant::now();
    let large = parse_source(
        &large_source,
        "occurrence_scale_large.dag",
        occurrence_id_allocator_initial(),
    );
    let large_elapsed = large_start.elapsed();
    let large_bytes = serde_json::to_vec(&large.occurrence_transport)
        .expect("serialize large occurrence transport")
        .len();
    let large_rss = max_rss_kib();

    assert!(small.result.error.is_none(), "{:?}", small.result.error);
    assert!(large.result.error.is_none(), "{:?}", large.result.error);
    assert!(large_bytes > small_bytes);
    assert!(
        large_bytes <= small_bytes * 3,
        "doubling source declarations must remain linear: small={small_bytes}, large={large_bytes}"
    );
    assert!(
        large_elapsed <= small_elapsed.saturating_mul(8) + Duration::from_secs(2),
        "doubling source declarations exceeded a generous linear-time wall: small={small_elapsed:?}, large={large_elapsed:?}"
    );
    assert!(
        large_rss <= small_rss.saturating_mul(2).max(small_rss + 64 * 1024),
        "doubling source declarations exceeded the RSS wall: small={small_rss}KiB, large={large_rss}KiB"
    );
}
