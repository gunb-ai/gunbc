// AUTO-GENERATED from `src/v3/lenses/lower_helpers.dag` via
// `emit_rust_module`. Regenerate instead of hand-editing.

pub fn expr_span(p0: &parse_surface::SurfaceExpr) -> SourceSpan {
    match p0 {
        SurfaceExpr::Literal {
            value: __e_value,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::Var {
            name: __e_name,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::Path {
            segments: __e_segments,
            segment_spans: __e_segment_spans,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::Call {
            target: __e_target,
            args: __e_args,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::VariantRecord {
            target: __e_target,
            fields: __e_fields,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::Operator {
            op: __e_op,
            args: __e_args,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::Lambda {
            params: __e_params,
            body: __e_body,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::If {
            cond: __e_cond,
            then_branch: __e_then_branch,
            else_branch: __e_else_branch,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::Match {
            scrutinee: __e_scrutinee,
            arms: __e_arms,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::Record {
            fields: __e_fields,
            span: __e_span,
        } => (__e_span).clone(),
        SurfaceExpr::List {
            elements: __e_elements,
            span: __e_span,
        } => (__e_span).clone(),
    }
}
