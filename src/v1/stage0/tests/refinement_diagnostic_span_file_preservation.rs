use v1_compiler::v1_std_core::{kernel_span, make_file_span, no_span};

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

// The fileless constructor takes no offsets, so it cannot report a located range inside a
// file that does not exist. This is the executing half of that claim: whatever no_span()
// renders its absent file as, its range is always empty. The deleted make_span(start, end)
// is what made the other combination -- fabricated file, caller-supplied offsets --
// expressible; nothing here can reconstruct it.
#[test]
fn no_span_carries_no_offsets() {
    let span = no_span();

    assert_eq!(span.start, 0, "the null span must carry no start offset");
    assert_eq!(span.end, 0, "the null span must carry no end offset");
}

#[test]
fn no_span_is_distinguishable_from_an_authored_span() {
    let null_span = no_span();
    let authored = make_file_span("actual.dag".to_string(), 0, 10);

    assert_ne!(
        null_span.file, authored.file,
        "the null span must not be mistakable for a span authored in a real file"
    );
    assert_eq!(
        authored.file, "actual.dag",
        "make_file_span file must be preserved"
    );
}

// kernel_span is the third and last span constructor: it names the kernel entity as its
// file rather than fabricating a source file, and its range is derived from that name, not
// supplied by the caller.
#[test]
fn kernel_span_names_its_kernel_entity_and_derives_its_range() {
    let span = kernel_span("Unit".to_string());

    assert_eq!(span.file, "<kernel:Unit>");
    assert_eq!(span.start, 0);
    assert_eq!(
        span.end, 4,
        "kernel span range is derived from the name length"
    );
}
