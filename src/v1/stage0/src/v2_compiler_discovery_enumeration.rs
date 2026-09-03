// Seed realization for v2.compiler.discovery_enumeration (Wave 2 Band A).
// Hand-retained Rust oracle — independent of the self-emitted artifact under test.
// Dissolve-on: self-emit cutover retires this module when v2.compiler.discovery_enumeration
// is emitted-only and the behavioral harness is modeled (sde_scaffold_dissolution_trigger).

pub type Symbol = String;

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct ResolvedDeclRef {
    pub module: String,
    pub name: Symbol,
}

pub fn unified_claim_arm_bool_witness_claim_module() -> String {
    "v2.std.verification".to_string()
}

pub fn unified_claim_arm_node_corpus_module() -> String {
    "v2.std.verification".to_string()
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
#[serde(tag = "_variant")]
pub enum OwnedDataDeclInitializer {
    OwnedBoolWitnessClaimInit {
        witness_entry: String,
        witness_function: String,
    },
    OwnedNodeCorpusInit,
    OwnedOtherInit {
        resolved: ResolvedDeclRef,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OwnedDataDeclRecord {
    pub entry: String,
    pub module: String,
    pub decl_name: String,
    pub initializer: OwnedDataDeclInitializer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, serde::Serialize, serde::Deserialize)]
pub struct OwnedDataDiscoveryReceipt {
    pub unified_claim_arm_count: i64,
    pub bool_witness_claim_arm_count: i64,
    pub illegal_other_init_count: i64,
    pub bool_witness_transport_row_count: i64,
    pub transport_projection_complete: bool,
}
