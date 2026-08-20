use v1_compiler::v1_std_core::{make_file_span, make_span};

#[test]
fn make_file_span_preserves_file_name() {
    let real_file = "mymodule.dag";
    let span = make_file_span(real_file.to_string(), 42, 100);

    assert_eq!(
        span.file, real_file,
        "make_file_span must preserve the provided file name"
    );
    assert_eq!(span.start, 42, "make_file_span must preserve start offset");
    assert_eq!(span.end, 100, "make_file_span must preserve end offset");
}

#[test]
fn make_span_hardcodes_synthetic_file() {
    let span = make_span(42, 100);

    assert_eq!(
        span.file, "<synthetic>",
        "make_span correctly hardcodes file to <synthetic> for truly synthetic spans"
    );
    assert_eq!(span.start, 42, "make_span must preserve start offset");
    assert_eq!(span.end, 100, "make_span must preserve end offset");
}

#[test]
fn make_file_span_distinct_from_make_span() {
    let span1 = make_span(0, 10);
    let span2 = make_file_span("actual.dag".to_string(), 0, 10);

    assert_ne!(
        span1.file, span2.file,
        "make_file_span and make_span must produce different file fields"
    );
    assert_eq!(
        span1.file, "<synthetic>",
        "make_span file must be synthetic"
    );
    assert_eq!(
        span2.file, "actual.dag",
        "make_file_span file must be preserved"
    );
}
