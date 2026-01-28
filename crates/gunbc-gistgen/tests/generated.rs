//! Generated tests from SetSpec declarations.
//!
//! This file demonstrates the SetSpec-based test generation pattern.
//! Tests are derived from type contracts, not written manually.

use gunbc_gistgen::setspec::{
    AuthSpec, BuildGistRequestAccepts, BuildGistRequestProduces, EnumerateFilesProduces,
    FilterFilesAccepts, FilterFilesProduces, GistApiAccepts, GistApiSpec, ReadFilesAccepts,
    ReadFilesProduces, RepoFilesSpec,
};
use gunbc_test::{
    check_composition, generate_permutations, Cardinality, IntegrationIssue, SetSpec,
    SetSpecOutput,
};

// =============================================================================
// Single-type tests: verify each type's cardinality behavior
// =============================================================================

#[test]
fn repo_files_spec_covers_all_cardinalities() {
    let cases = RepoFilesSpec::cases();
    let cardinalities: Vec<_> = cases.iter().map(|c| c.cardinality).collect();

    assert!(cardinalities.contains(&Cardinality::Zero));
    assert!(cardinalities.contains(&Cardinality::One));
    assert!(cardinalities.contains(&Cardinality::N));
    assert!(cardinalities.contains(&Cardinality::Null));
}

#[test]
fn gist_api_spec_covers_all_cardinalities() {
    let cases = GistApiSpec::cases();
    let cardinalities: Vec<_> = cases.iter().map(|c| c.cardinality).collect();

    assert!(cardinalities.contains(&Cardinality::Zero));
    assert!(cardinalities.contains(&Cardinality::One));
    assert!(cardinalities.contains(&Cardinality::N));
    assert!(cardinalities.contains(&Cardinality::Null));
}

#[test]
fn auth_spec_covers_upsert_cases() {
    let cases = AuthSpec::cases();
    // Auth is a binary upsert: exists (One) or needs creation (Zero)
    let cardinalities: Vec<_> = cases.iter().map(|c| c.cardinality).collect();

    assert!(cardinalities.contains(&Cardinality::One)); // token exists
    assert!(cardinalities.contains(&Cardinality::Zero)); // token missing, create
}

// =============================================================================
// Composition tests: verify permutations generate expected outcomes
// =============================================================================

#[test]
fn repo_files_gist_api_permutations() {
    let perms = generate_permutations::<RepoFilesSpec, GistApiSpec>();

    // Should have 4 * 4 = 16 permutations
    assert_eq!(perms.len(), 16);

    // Check specific composition behaviors:
    for (repo, gist) in &perms {
        match (repo.cardinality, gist.cardinality) {
            // Zero files from repo -> GistApi should error on empty
            (Cardinality::Zero, _) => {
                // When repo produces 0 files, gist receives empty request
                // GistApi.Zero case applies -> error expected
            }
            // One file from repo -> GistApi.One -> success
            (Cardinality::One, Cardinality::One) => {
                assert!(matches!(gist.expected, SetSpecOutput::Ok(_)));
            }
            // N files from repo -> GistApi.N -> success
            (Cardinality::N, Cardinality::N) => {
                assert!(matches!(gist.expected, SetSpecOutput::Ok(_)));
            }
            // Null from repo -> propagates error
            (Cardinality::Null, _) => {
                assert!(matches!(repo.expected, SetSpecOutput::Err(_)));
            }
            _ => {}
        }
    }
}

// =============================================================================
// Invariant tests: verify SetSpec contracts are internally consistent
// =============================================================================

#[test]
fn gist_api_zero_always_errors() {
    let cases = GistApiSpec::cases();
    let zero_case = cases
        .iter()
        .find(|c| c.cardinality == Cardinality::Zero)
        .unwrap();
    assert!(matches!(zero_case.expected, SetSpecOutput::Err(_)));
}

#[test]
fn gist_api_null_always_errors() {
    let cases = GistApiSpec::cases();
    let null_case = cases
        .iter()
        .find(|c| c.cardinality == Cardinality::Null)
        .unwrap();
    assert!(matches!(null_case.expected, SetSpecOutput::Err(_)));
}

#[test]
fn repo_files_null_always_errors() {
    let cases = RepoFilesSpec::cases();
    let null_case = cases
        .iter()
        .find(|c| c.cardinality == Cardinality::Null)
        .unwrap();
    assert!(matches!(null_case.expected, SetSpecOutput::Err(_)));
}

// =============================================================================
// Composition checking: detect integration bugs between adjacent nodes
// =============================================================================

/// The core integration test: ReadFiles → BuildGistRequest composition.
///
/// This catches the real bug:
/// - ReadFiles can produce Zero (empty map when no files match)
/// - BuildGistRequest rejects Zero (can't create empty gist)
/// - Without SetSpec: silent failure or runtime error
/// - With SetSpec: test reveals this as a known edge case
#[test]
fn read_files_to_build_gist_request_composition() {
    let result = check_composition::<ReadFilesProduces, BuildGistRequestAccepts>();

    // Should find no unhandled bugs
    assert!(
        result.is_ok(),
        "Unexpected bugs: {:?}",
        result.bugs
    );

    // Should find the Zero edge case
    let zero_edge = result.edge_cases.iter().find(|e| e.cardinality == Cardinality::Zero);
    assert!(
        zero_edge.is_some(),
        "Expected Zero edge case: ReadFiles can produce Zero but BuildGistRequest rejects it"
    );

    // Verify it's marked as a known rejection
    let zero_edge = zero_edge.unwrap();
    assert_eq!(zero_edge.from, "ReadFiles");
    assert_eq!(zero_edge.to, "BuildGistRequest");
    assert_eq!(zero_edge.issue, IntegrationIssue::KnownRejection);
}

/// EnumerateFiles → FilterFiles composition.
///
/// FilterFiles accepts all cardinalities that EnumerateFiles can produce,
/// so this should have no bugs or edge cases (except Null handling).
#[test]
fn enumerate_to_filter_composition() {
    let result = check_composition::<EnumerateFilesProduces, FilterFilesAccepts>();

    // Should find no unhandled bugs
    assert!(
        result.is_ok(),
        "Unexpected bugs: {:?}",
        result.bugs
    );

    // No edge cases for valid paths (Null errors in EnumerateFiles, doesn't flow)
    assert!(
        result.edge_cases.is_empty(),
        "Unexpected edge cases: {:?}",
        result.edge_cases
    );
}

/// FilterFiles → ReadFiles composition.
///
/// Both accept the same cardinalities, so this should be clean.
#[test]
fn filter_to_read_composition() {
    let result = check_composition::<FilterFilesProduces, ReadFilesAccepts>();

    assert!(
        result.is_ok(),
        "Unexpected bugs: {:?}",
        result.bugs
    );
    assert!(
        result.edge_cases.is_empty(),
        "Unexpected edge cases: {:?}",
        result.edge_cases
    );
}

/// BuildGistRequest → GistApi composition.
///
/// BuildGistRequest only produces One (for valid inputs), and GistApi accepts One/N.
#[test]
fn build_request_to_gist_api_composition() {
    let result = check_composition::<BuildGistRequestProduces, GistApiAccepts>();

    assert!(
        result.is_ok(),
        "Unexpected bugs: {:?}",
        result.bugs
    );
    assert!(
        result.edge_cases.is_empty(),
        "Unexpected edge cases: {:?}",
        result.edge_cases
    );
}

/// Full pipeline composition check.
///
/// Verifies the entire gistgen data flow:
/// ```text
/// EnumerateFiles → FilterFiles → ReadFiles → BuildGistRequest → GistApi
/// ```
#[test]
fn full_pipeline_composition() {
    // Each stage in the pipeline
    let stage1 = check_composition::<EnumerateFilesProduces, FilterFilesAccepts>();
    let stage2 = check_composition::<FilterFilesProduces, ReadFilesAccepts>();
    let stage3 = check_composition::<ReadFilesProduces, BuildGistRequestAccepts>();
    let stage4 = check_composition::<BuildGistRequestProduces, GistApiAccepts>();

    // Collect all bugs
    let all_bugs: Vec<_> = [&stage1, &stage2, &stage3, &stage4]
        .iter()
        .flat_map(|r| r.bugs.iter())
        .collect();

    assert!(
        all_bugs.is_empty(),
        "Pipeline has unhandled integration bugs: {:?}",
        all_bugs
    );

    // Collect all edge cases
    let all_edges: Vec<_> = [&stage1, &stage2, &stage3, &stage4]
        .iter()
        .flat_map(|r| r.edge_cases.iter())
        .collect();

    // We expect exactly one edge case: ReadFiles.Zero → BuildGistRequest
    assert_eq!(
        all_edges.len(),
        1,
        "Expected exactly 1 edge case, got: {:?}",
        all_edges
    );
    assert_eq!(all_edges[0].from, "ReadFiles");
    assert_eq!(all_edges[0].to, "BuildGistRequest");
}

// =============================================================================
// Negative tests: verify composition checker catches intentional mismatches
// =============================================================================

/// A deliberately broken ProducesSpec that claims it never produces Zero.
/// Used to verify the composition checker would catch this lie.
mod broken_specs {
    use gunbc_test::{Cardinality, ProducesCase, ProducesSpec};

    pub struct BrokenReadFilesProduces;

    impl ProducesSpec for BrokenReadFilesProduces {
        fn produces() -> Vec<(Cardinality, ProducesCase)> {
            // LIE: claims it never produces Zero
            vec![
                (Cardinality::One, ProducesCase::Ok(Cardinality::One)),
                (Cardinality::N, ProducesCase::Ok(Cardinality::N)),
                (Cardinality::Null, ProducesCase::Err),
            ]
        }

        fn name() -> &'static str {
            "BrokenReadFiles"
        }
    }
}

#[test]
fn broken_spec_hides_edge_case() {
    use broken_specs::BrokenReadFilesProduces;

    let result = check_composition::<BrokenReadFilesProduces, BuildGistRequestAccepts>();

    // The broken spec hides the Zero edge case
    assert!(result.edge_cases.is_empty());
    assert!(result.is_ok());

    // In a real codebase, this would be caught by integration tests
    // that actually run the operations and observe Zero being produced.
}
