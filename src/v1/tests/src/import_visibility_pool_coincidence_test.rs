//! Fixture-only control for the pool-coincidental import-visibility false green.
//!
//! Loading a provider must make its declaration data available to resolution without
//! making every declaration visible to every consumer. Today the latter still happens:
//! a selective import loads the provider, after which a declaration omitted from the
//! authored name list resolves with only an `UnlistedImportUse` advisory.
//!
//! The ignored tests state the intended refusal and deliberately fail on the current
//! resolver. Wave B will remove `#[ignore]` when it installs the visibility wall. The
//! active false-green receipt keeps the defect observable in the meantime, while the
//! positive arm proves an eventual refusal is about visibility rather than fixture
//! breakage.

use crate::helpers::{compile_multi, diagnostic_messages};

const PROVIDER: &str = "module visibility_fixture.provider\n\
    data Imported: Int = 1\n\
    data NotImported: Int = 2\n";

const HOMONYM_PROVIDER: &str = "module visibility_fixture.homonym_provider\n\
    data HomonymAnchor: Int = 3\n\
    data NotImported: Int = 4\n";

const DIRECT_IMPORT_BRIDGE: &str = "module visibility_fixture.bridge\n\
    import visibility_fixture.provider { NotImported }\n\
    data BridgeAnchor: Int = NotImported\n";

fn hard_messages(files: &[(&str, &str)]) -> Vec<String> {
    diagnostic_messages(&compile_multi(files))
        .into_iter()
        .filter(|message| {
            !message.starts_with("complexity: ") && !message.starts_with("unlisted import use ")
        })
        .collect()
}

fn expects_not_imported_refusal(files: &[(&str, &str)]) {
    let messages = diagnostic_messages(&compile_multi(files));
    assert!(
        messages.iter().any(|message| {
            message.contains("NotImported")
                && (message.contains("unbound")
                    || message.contains("not imported")
                    || message.contains("undefined variable"))
        }),
        "a located refusal must name the non-visible `NotImported` declaration; got \
         {messages:?}"
    );
}

#[test]
fn omitted_declaration_currently_false_greens_from_loaded_pool() {
    let consumer = "module visibility_fixture.selective_consumer\n\
        import visibility_fixture.provider { Imported }\n\
        import visibility_fixture.homonym_provider { HomonymAnchor }\n\
        data observed: Int = NotImported\n";
    let files = [
        ("provider.dag", PROVIDER),
        ("homonym_provider.dag", HOMONYM_PROVIDER),
        ("consumer.dag", consumer),
    ];
    let result = compile_multi(&files);
    let messages = diagnostic_messages(&result);

    assert!(
        hard_messages(&files).is_empty(),
        "FALSE GREEN: an omitted declaration currently resolves because its provider is \
         loaded; the second provider must not turn that into pool-wide ambiguity: \
         {messages:?}"
    );
    assert!(
        messages.iter().any(|message| {
            message.starts_with("unlisted import use") && message.contains("NotImported")
        }),
        "the false green must remain visible as an UnlistedImportUse advisory: {messages:?}"
    );
}

#[test]
fn explicitly_importing_both_declarations_is_green() {
    let consumer = "module visibility_fixture.explicit_consumer\n\
        import visibility_fixture.provider { Imported, NotImported }\n\
        import visibility_fixture.homonym_provider { HomonymAnchor }\n\
        data observed: Int = NotImported\n";
    let files = [
        ("provider.dag", PROVIDER),
        ("homonym_provider.dag", HOMONYM_PROVIDER),
        ("consumer.dag", consumer),
    ];

    assert!(
        hard_messages(&files).is_empty(),
        "the explicitly imported declaration must remain visible"
    );
}

#[test]
#[ignore = "known red: Wave B must make loaded declarations non-visible unless imported"]
fn omitted_declaration_is_located_refusal_even_with_pool_homonym() {
    let consumer = "module visibility_fixture.selective_consumer\n\
        import visibility_fixture.provider { Imported }\n\
        import visibility_fixture.homonym_provider { HomonymAnchor }\n\
        data observed: Int = NotImported\n";
    expects_not_imported_refusal(&[
        ("provider.dag", PROVIDER),
        ("homonym_provider.dag", HOMONYM_PROVIDER),
        ("consumer.dag", consumer),
    ]);
}

#[test]
#[ignore = "known red: Wave B must prevent transitive imports from conferring visibility"]
fn direct_imports_are_not_transitive() {
    let consumer = "module visibility_fixture.chain_consumer\n\
        import visibility_fixture.bridge { BridgeAnchor }\n\
        data observed: Int = NotImported\n";
    expects_not_imported_refusal(&[
        ("provider.dag", PROVIDER),
        ("bridge.dag", DIRECT_IMPORT_BRIDGE),
        ("consumer.dag", consumer),
    ]);
}
