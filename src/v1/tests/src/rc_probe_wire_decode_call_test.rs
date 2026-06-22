use crate::helpers::read_v2_file;

const TARGET_MODEL_DAG: &str = "src/v2/std/compilers/target_model.dag";

fn live_source(source: &str) -> String {
    source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn probe_wire_decode_call_uses_node_and_projection_args() {
    let source = read_v2_file(TARGET_MODEL_DAG);
    let live = live_source(&source);

    assert!(
        live.contains(
            "target_type_expr_emitted_wire_decode(node: emitted, projection: projection)"
        ),
        "SG-RC probe must call target_type_expr_emitted_wire_decode with (node:, projection:) \
         (ctrl#1476 B3-Gap2). target_model.dag:target_reference_layer_probe_from_emitted_type."
    );

    assert!(
        !live.contains("target_type_expr_emitted_wire_decode(emitted:"),
        "regression: target_type_expr_emitted_wire_decode called with `emitted:` — its signature is \
         (node:, projection:); the misnamed arg leaves `node` unbound at runtime \
         ('undefined variable: node'). ctrl#1476 B3-Gap2."
    );
}
