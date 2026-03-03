//! Make target modeling — delegates to gunbc_codegen::makegen::model.

pub use gunbc_codegen::makegen::model::{
    index_unique_target_names, load_build_targets_data, validate_target_namespace,
    validate_target_namespace_with_data, BuildTargetsData, CoreWorkflowData, MakegenModelError,
    MetaTargetData, ResourceNeedData, ResourceTargetEntryData, TargetOrigin, TargetSource,
};
