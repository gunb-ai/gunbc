//! Fixture-only control for the pool-coincidental import-visibility false green.
//!
//! Loading a provider must make its declaration data available to resolution without
//! making every declaration visible to every consumer. Today the latter still happens:
//! a selective import loads the provider, after which a declaration omitted from the
//! authored name list resolves silently. An unrelated module loading the provider also
//! makes that declaration resolve in a consumer with no provider import edge at all.
//!
//! Every row executes against the current resolver. The false-green rows assert today's
//! leniency, so Wave B's visibility wall will make them fail and force their assertions
//! to flip to located refusals. The positive arm proves an eventual refusal is about
//! visibility rather than fixture breakage.
//!
//! SCAFFOLD (DESIGN §7 HAND-RUST GATE — explicit deferral): this module exercises the
//! v1 compiler-test harness because the deliberately invalid consumer sources cannot be
//! committed beneath a discovered `.dag` root before the visibility wall exists. Lane:
//! namespace, ROADMAP node `namespace-canonical-binding` (“Make name lookup itself
//! produce that one answer, and delete the old way”), under “Work out what each name can
//! see, and let that be what a file depends on.” Concrete dissolution: when that node's
//! production changeover makes the false-green rows refuse, flip them to located-refusal
//! assertions and delete this Rust module once the same fixture is enrolled as the
//! namespace lane's `.dag` closing witness; otherwise it deletes with the
//! `v1-test-migration` prerequisite of `v1-zero-hand-maintained-rust`.

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

const UNRELATED_LOADER: &str = "module visibility_fixture.unrelated_loader\n\
    import visibility_fixture.provider { Imported }\n\
    data LoaderValue: Int = Imported\n";

fn hard_messages(files: &[(&str, &str)]) -> Vec<String> {
    diagnostic_messages(&compile_multi(files))
        .into_iter()
        .filter(|message| {
            !message.starts_with("complexity: ") && !message.starts_with("unlisted import use ")
        })
        .collect()
}

fn asserts_current_not_imported_false_green(files: &[(&str, &str)], control: &str) {
    let messages = diagnostic_messages(&compile_multi(files));
    assert!(
        hard_messages(files).is_empty(),
        "FALSE GREEN pinned for Wave B: {control} currently resolves non-visible \
         `NotImported`; the visibility wall must make this assertion fail before it is \
         flipped to require a located refusal. Got {messages:?}"
    );
    assert!(
        !messages
            .iter()
            .any(|message| message.contains("NotImported")),
        "FALSE GREEN pinned for Wave B: {control} is currently silent; a diagnostic \
         unexpectedly named `NotImported`: {messages:?}"
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
        !messages
            .iter()
            .any(|message| message.contains("NotImported")),
        "the current false green is fully silent even with a pool homonym; a diagnostic \
         unexpectedly named `NotImported`: {messages:?}"
    );
}

#[test]
fn unrelated_module_loading_provider_creates_pool_coincidence_false_green() {
    let consumer = "module visibility_fixture.pool_consumer\n\
        data observed: Int = NotImported\n";

    let without_loader = diagnostic_messages(&compile_multi(&[("consumer.dag", consumer)]));
    assert!(
        without_loader
            .iter()
            .any(|message| message.contains("NotImported")),
        "RED control: without the unrelated loader, the consumer's bare declaration must \
         not resolve: {without_loader:?}"
    );

    let with_loader = [
        ("provider.dag", PROVIDER),
        ("unrelated_loader.dag", UNRELATED_LOADER),
        ("consumer.dag", consumer),
    ];
    let messages = diagnostic_messages(&compile_multi(&with_loader));
    assert!(
        hard_messages(&with_loader).is_empty(),
        "FALSE GREEN: module C importing the provider for `Imported` currently makes \
         `NotImported` resolve in unrelated module B: {messages:?}"
    );
    assert!(
        !messages
            .iter()
            .any(|message| message.contains("NotImported")),
        "the unrelated-loader pool coincidence is currently a silent false green; got \
         a diagnostic naming `NotImported`: {messages:?}"
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
fn homonymous_provider_does_not_ambiguate_selective_import_false_green() {
    let consumer = "module visibility_fixture.selective_consumer\n\
        import visibility_fixture.provider { Imported }\n\
        import visibility_fixture.homonym_provider { HomonymAnchor }\n\
        data observed: Int = NotImported\n";
    asserts_current_not_imported_false_green(
        &[
            ("provider.dag", PROVIDER),
            ("homonym_provider.dag", HOMONYM_PROVIDER),
            ("consumer.dag", consumer),
        ],
        "a homonymous loaded provider neither refuses nor creates ambiguity",
    );
}

#[test]
fn direct_import_chain_currently_confers_transitive_visibility() {
    let consumer = "module visibility_fixture.chain_consumer\n\
        import visibility_fixture.bridge { BridgeAnchor }\n\
        data observed: Int = NotImported\n";
    asserts_current_not_imported_false_green(
        &[
            ("provider.dag", PROVIDER),
            ("bridge.dag", DIRECT_IMPORT_BRIDGE),
            ("consumer.dag", consumer),
        ],
        "a direct import by the bridge incorrectly confers transitive visibility",
    );
}
