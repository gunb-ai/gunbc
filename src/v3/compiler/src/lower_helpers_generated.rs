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
pub fn item_span(p0: &parse_surface::SurfaceItem) -> SourceSpan {
    match p0 {
        SurfaceItem::Let {
            name: __i_name,
            type_ann: __i_type_ann,
            expr: __i_expr,
        } => expr_span(__i_expr),
        SurfaceItem::Fn {
            name: __i_name,
            type_params: __i_type_params,
            params: __i_params,
            return_type: __i_return_type,
            body: __i_body,
            span: __i_span,
        } => (__i_span).clone(),
        SurfaceItem::FnExternalBody {
            name: __i_name,
            type_params: __i_type_params,
            params: __i_params,
            return_type: __i_return_type,
            body_span: __i_body_span,
            span: __i_span,
        } => (__i_span).clone(),
        SurfaceItem::Data {
            name: __i_name,
            ty: __i_ty,
            body: __i_body,
            body_span: __i_body_span,
            span: __i_span,
        } => (__i_span).clone(),
        SurfaceItem::Module {
            path: __i_path,
            span: __i_span,
        } => (__i_span).clone(),
        SurfaceItem::Import {
            path: __i_path,
            names: __i_names,
            span: __i_span,
        } => (__i_span).clone(),
        SurfaceItem::TypeAtom {
            name: __i_name,
            type_params: __i_type_params,
            span: __i_span,
        } => (__i_span).clone(),
        SurfaceItem::TypeRecord {
            name: __i_name,
            type_params: __i_type_params,
            fields: __i_fields,
            span: __i_span,
        } => (__i_span).clone(),
        SurfaceItem::TypeSum {
            name: __i_name,
            type_params: __i_type_params,
            variants: __i_variants,
            inhabits: __i_inhabits,
            span: __i_span,
        } => (__i_span).clone(),
        SurfaceItem::TypeAlias {
            name: __i_name,
            type_params: __i_type_params,
            target: __i_target,
            refinement: __i_refinement,
            span: __i_span,
        } => (__i_span).clone(),
    }
}
pub fn pattern_binding_names(p0: &parse_surface::SurfacePattern) -> Vec<String> {
    match p0 {
        SurfacePattern::BareVariant { name: _, span: _ } => Vec::new(),
        SurfacePattern::VariantWith {
            name: __v_name,
            binding: __v_binding,
            span: __v_span,
        } => vec![(__v_binding).clone()],
        SurfacePattern::VariantFields {
            name: __v_name,
            fields: __v_fields,
            span: __v_span,
        } => pattern_field_bindings(__v_fields),
    }
}
pub fn pattern_field_bindings(p0: &[parse_surface::SurfacePatternField]) -> Vec<String> {
    match p0 {
        [] => Vec::new(),
        [__list_head, __list_tail @ ..] => {
            let mut __list = pattern_field_bindings(__list_tail);
            __list.insert(0, ((__list_head).binding).clone());
            __list
        }
    }
}
