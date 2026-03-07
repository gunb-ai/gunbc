//! End-to-end fidelity smoke tests against real DSL modules.
//!
//! The classification engine itself lives in `gunbc_codegen::fidelity`.
//! These tests exercise it against the real DSL corpus via `gunbc_resolve`.

#[cfg(test)]
mod tests {
    use gunbc_codegen::fidelity::{classify_callable, classify_module, FidelityClassification};
    use gunbc_resolve::{builder::build_dsl_graph, BuildOpts};
    use gunbc_test::FermiCost;
    use gunbc_test::TestClass;

    fn classify_dsl_module(module_path: &str) -> FidelityClassification {
        let result = build_dsl_graph(
            module_path,
            crate::extern_ops::gunbc_runtime_bindings(),
            BuildOpts::default(),
        )
        .unwrap_or_else(|e| panic!("DSL module `{module_path}` should compile: {e}"));
        classify_module(&result.callable_properties)
    }

    #[test]
    fn classify_makegen_callable_is_unit_xs() {
        // makegen itself has no transport operations (pure DSL rendering + content_upsert).
        // Module-level classification includes transitive auth callables from std.patterns,
        // but the makegen callable specifically should be Unit/XS.
        let result = build_dsl_graph(
            "tools/makegen.dag",
            crate::extern_ops::gunbc_runtime_bindings(),
            BuildOpts::default(),
        )
        .expect("makegen should compile");
        let makegen_props = result
            .callable_properties
            .get("tools.makegen::makegen")
            .expect("makegen callable should exist in properties");
        let classification = classify_callable(makegen_props);
        assert_eq!(classification.test_class, TestClass::Unit);
        assert_eq!(classification.fermi_cost, FermiCost::XS);
        assert!(classification.hermetic);
    }

    #[test]
    fn classify_gist_module_is_integration_l() {
        let classification = classify_dsl_module("tools/gist.dag");
        assert_eq!(classification.test_class, TestClass::Integration);
        assert_eq!(classification.fermi_cost, FermiCost::L);
        assert!(!classification.hermetic);
    }
}
