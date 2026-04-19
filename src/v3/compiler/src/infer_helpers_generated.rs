// AUTO-GENERATED from `src/v3/lenses/infer_helpers.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

pub fn behavior_output_port(p0: &Behavior) -> PortId {
    match p0 {
        Behavior::Value(v) => (v).result_port(),
        Behavior::Transform(t) => (t).result_port(),
        Behavior::Branch(b) => (b).result_port(),
        Behavior::Loop(l) => (l).result_port(),
        Behavior::Bind(bind) => (bind).result_port(),
    }
}
