//! Unused-import analysis for `.dag` source files.
//!
//! Walks a parsed `SourceFile` AST structurally (no text scanning) and
//! reports import bindings that are never referenced in the module body.
//! Module-level imports (no explicit bindings) can also be checked
//! against an export index derived from the resolved module graph.

use std::collections::{HashMap, HashSet};

use daglang_syntax::ast::{
    CapabilityDef, Expr, Field, ForBody, Item, MatchArm, OperationDef, Pattern, SourceFile, Stmt,
    TransportBinding, TypeBody, TypeExpr, UsesClause,
};

use crate::ResolvedModule;

/// A single unused import binding or module-level import.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnusedImport {
    /// Dotted module path of the import (e.g. `"std.types"`).
    pub module_path: String,
    /// The specific unused binding name, or `None` for a module-level import.
    pub binding: Option<String>,
}

/// Exported-name lookup keyed by dotted module path.
pub type ModuleExportIndex = HashMap<String, HashSet<String>>;

/// Find all unused imports in a parsed source file.
///
/// An import binding is "unused" if its name never appears in any type
/// annotation, expression, pattern, or declaration in the module body.
/// Module-level imports (without explicit bindings) are only checked for
/// aliased imports; non-aliased module imports require an export index
/// (see [`find_unused_imports_with_export_index`]) for accurate results
/// and are conservatively treated as used when no index is available.
pub fn find_unused_imports(source: &SourceFile) -> Vec<UnusedImport> {
    find_unused_imports_inner(source, None)
}

/// Build a one-pass index of names exported by each resolved module.
///
/// This lets module-level unused-import analysis match imports against the
/// names the imported module actually exports, including dotted service
/// namespaces like `gcp.STS` and `github.Gist`.
pub fn build_module_export_index(modules: &[ResolvedModule]) -> ModuleExportIndex {
    modules
        .iter()
        .map(|module| {
            (
                module.module_path.as_dotted(),
                collect_exported_names(&module.ast),
            )
        })
        .collect()
}

/// Find all unused imports using a precomputed module export index.
///
/// Non-aliased module-level imports are matched exclusively against the
/// exported service and type namespaces in the index. This replaces the
/// prior heuristic of matching the final path segment.
pub fn find_unused_imports_with_export_index(
    source: &SourceFile,
    export_index: &ModuleExportIndex,
) -> Vec<UnusedImport> {
    find_unused_imports_inner(source, Some(export_index))
}

fn find_unused_imports_inner(
    source: &SourceFile,
    export_index: Option<&ModuleExportIndex>,
) -> Vec<UnusedImport> {
    let mut referenced = HashSet::new();
    collect_all_referenced_names(source, &mut referenced);

    let mut unused = Vec::new();
    for import in &source.imports {
        let path_str = import.node.path.as_dotted();
        match &import.node.bindings {
            Some(bindings) => {
                for binding in bindings {
                    if !referenced.contains(binding.as_str()) {
                        unused.push(UnusedImport {
                            module_path: path_str.clone(),
                            binding: Some(binding.clone()),
                        });
                    }
                }
            }
            None => {
                let path_str = import.node.path.as_dotted();
                let is_used = if let Some(alias) = import.node.alias.as_deref() {
                    // Aliased module import: the alias is structural.
                    referenced.contains(alias)
                } else if let Some(index) = export_index {
                    // Non-aliased: match against exported service/type namespaces.
                    index
                        .get(&path_str)
                        .is_some_and(|exports| {
                            exports.iter().any(|name| referenced.contains(name))
                        })
                } else {
                    // No export index: cannot verify structurally, skip.
                    true
                };
                if !is_used {
                    unused.push(UnusedImport {
                        module_path: path_str,
                        binding: None,
                    });
                }
            }
        }
    }
    unused
}

fn collect_exported_names(source: &SourceFile) -> HashSet<String> {
    let mut exported = HashSet::new();
    for item in &source.items {
        collect_exported_item_names(&item.node, &mut exported);
    }
    exported
}

fn collect_exported_item_names(item: &Item, names: &mut HashSet<String>) {
    match item {
        Item::TypeDef(def) => {
            names.insert(def.name.clone());
            collect_variant_exports(&def.body, names);
        }
        Item::FnDef(def) => {
            names.insert(def.name.clone());
        }
        Item::FuncDef(def) => {
            names.insert(def.name.clone());
        }
        Item::PatternDef(def) => {
            names.insert(def.name.clone());
        }
        Item::ServiceDef(def) => {
            names.insert(def.name.clone());
        }
        Item::ResourceDef(def) => {
            names.insert(def.name.clone());
        }
        Item::InterfaceDef(def) => {
            names.insert(def.name.clone());
        }
        Item::PipelineDef(def) => {
            names.insert(def.name.clone());
        }
        Item::ProfileDef(def) => {
            names.insert(def.name.clone());
        }
        Item::ProjectDef(def) => {
            names.insert(def.name.clone());
        }
        Item::FeatureDef(def) => {
            names.insert(def.name.clone());
        }
        Item::TaskDef(def) => {
            names.insert(def.name.clone());
        }
        Item::DesignDef(def) => {
            names.insert(def.name.clone());
        }
        Item::ComponentDef(def) => {
            names.insert(def.name.clone());
        }
        Item::EnvironmentDef(def) => {
            names.insert(def.name.clone());
        }
        Item::ParamDecl(decl) => {
            names.insert(decl.name.clone());
        }
        Item::DataDef(def) => {
            names.insert(def.name.clone());
        }
        Item::ExternAssetDecl(decl) => {
            names.insert(decl.name.clone());
        }
        Item::TestDef(_) | Item::FixtureDef(_) => {}
    }
}

fn collect_variant_exports(body: &TypeBody, names: &mut HashSet<String>) {
    if let TypeBody::Sum(variants) = body {
        names.extend(variants.iter().map(|variant| variant.name.clone()));
    }
}

// ── Name collection ──────────────────────────────────────────────────

fn collect_all_referenced_names(source: &SourceFile, names: &mut HashSet<String>) {
    for item in &source.items {
        collect_item_names(&item.node, names);
    }
}

fn collect_item_names(item: &Item, names: &mut HashSet<String>) {
    match item {
        Item::TypeDef(def) => {
            collect_type_body_names(&def.body, names);
        }
        Item::FnDef(def) => {
            collect_params_names(&def.params, names);
            collect_type_names(&def.return_type, names);
            collect_stmts_names(&def.body.stmts, names);
        }
        Item::FuncDef(def) => {
            collect_params_names(&def.params, names);
            collect_fields_names(&def.outputs, names);
            collect_uses_names(&def.uses, names);
            collect_provides_names(&def.provides, names);
            collect_stmts_names(&def.body.stmts, names);
        }
        Item::PatternDef(def) => {
            collect_params_names(&def.params, names);
            collect_fields_names(&def.outputs, names);
            collect_uses_names(&def.uses, names);
            collect_provides_names(&def.provides, names);
            collect_stmts_names(&def.body.stmts, names);
        }
        Item::ServiceDef(def) => {
            if let Some(iface) = &def.implements {
                names.insert(iface.clone());
            }
            for op in &def.operations {
                collect_operation_names(op, names);
            }
            for field in &def.config.extra {
                collect_type_names(&field.ty, names);
                if let Some(default) = &field.default {
                    collect_expr_names(default, names);
                }
            }
        }
        Item::ResourceDef(def) => {
            if let Some(iface) = &def.implements {
                names.insert(iface.clone());
            }
            for (_, expr) in &def.properties {
                collect_expr_names(expr, names);
            }
            collect_fields_names(&def.config, names);
            if let Some(acquire) = &def.acquire {
                collect_stmts_names(&acquire.stmts, names);
            }
            if let Some(release) = &def.release {
                collect_stmts_names(&release.stmts, names);
            }
            for cap in &def.capabilities {
                collect_capability_names(cap, names);
            }
        }
        Item::InterfaceDef(def) => {
            for cap in &def.capabilities {
                collect_capability_names(cap, names);
            }
            for type_def in &def.type_defs {
                collect_type_body_names(&type_def.body, names);
            }
        }
        Item::PipelineDef(def) => {
            collect_uses_names(&def.uses, names);
            for stage in &def.stages {
                collect_stmts_names(&stage.body.stmts, names);
                if let Some(when) = &stage.when {
                    collect_expr_names(when, names);
                }
            }
        }
        Item::ProfileDef(def) => {
            for bind in &def.binds {
                names.insert(bind.interface_type.clone());
                names.insert(bind.implementation_type.clone());
                for (_, expr) in &bind.config_entries {
                    collect_expr_names(expr, names);
                }
            }
        }
        Item::TestDef(def) => {
            for let_decl in &def.lets {
                collect_expr_names(&let_decl.value, names);
            }
            for mock in &def.mocks {
                collect_expr_names(&mock.value, names);
            }
            for input in &def.inputs {
                collect_expr_names(&input.value, names);
            }
            for expect in &def.expects {
                collect_expect_names(expect, names);
            }
        }
        Item::FixtureDef(def) => {
            for mock in &def.mocks {
                collect_expr_names(&mock.value, names);
            }
        }
        Item::ProjectDef(def) => {
            collect_property_exprs(&def.properties, names);
        }
        Item::FeatureDef(def) => {
            collect_property_exprs(&def.properties, names);
        }
        Item::TaskDef(def) => {
            collect_property_exprs(&def.properties, names);
        }
        Item::DesignDef(def) => {
            collect_property_exprs(&def.properties, names);
        }
        Item::ComponentDef(def) => {
            collect_property_exprs(&def.properties, names);
        }
        Item::EnvironmentDef(def) => {
            collect_property_exprs(&def.properties, names);
        }
        Item::ParamDecl(decl) => {
            collect_type_names(&decl.ty, names);
            if let Some(default) = &decl.default {
                collect_expr_names(default, names);
            }
        }
        Item::DataDef(def) => {
            collect_type_names(&def.ty, names);
            collect_expr_names(&def.value, names);
        }
        Item::ExternAssetDecl(decl) => {
            collect_type_names(&decl.ty, names);
        }
    }
}

fn collect_property_exprs(properties: &[(String, Expr)], names: &mut HashSet<String>) {
    for (_, expr) in properties {
        collect_expr_names(expr, names);
    }
}

fn collect_type_body_names(body: &TypeBody, names: &mut HashSet<String>) {
    match body {
        TypeBody::Record(fields) => collect_fields_names(fields, names),
        TypeBody::Sum(variants) => {
            for variant in variants {
                collect_fields_names(&variant.fields, names);
            }
        }
        TypeBody::Alias(ty) => collect_type_names(ty, names),
    }
}

fn collect_type_names(ty: &TypeExpr, names: &mut HashSet<String>) {
    match ty {
        TypeExpr::Named(name) => {
            // The parser may embed config/suffix in the name
            // (e.g., "Filesystem(mode:ReadWrite)"). Extract the base name.
            let base = name.split('(').next().unwrap_or(name).trim();
            names.insert(base.to_string());
        }
        TypeExpr::Generic(name, args) => {
            names.insert(name.clone());
            for arg in args {
                collect_type_names(arg, names);
            }
        }
        TypeExpr::AssociatedOutput(base) => {
            names.insert(base.clone());
        }
        TypeExpr::Function(params, output) => {
            for p in params {
                collect_type_names(p, names);
            }
            collect_type_names(output, names);
        }
        TypeExpr::Optional(inner) | TypeExpr::Refined(inner, _) => {
            collect_type_names(inner, names);
        }
        TypeExpr::Record(fields) => {
            collect_fields_names(fields, names);
        }
    }
}

fn collect_fields_names(fields: &[Field], names: &mut HashSet<String>) {
    for field in fields {
        collect_type_names(&field.ty, names);
        if let Some(default) = &field.default {
            collect_expr_names(default, names);
        }
    }
}

fn collect_params_names(params: &[daglang_syntax::ast::Param], names: &mut HashSet<String>) {
    for param in params {
        collect_type_names(&param.ty, names);
        if let Some(default) = &param.default {
            collect_expr_names(default, names);
        }
    }
}

fn collect_uses_names(uses: &[UsesClause], names: &mut HashSet<String>) {
    for u in uses {
        collect_type_names(&u.resource_type, names);
        if let Some(config) = &u.config {
            for (_, expr) in config {
                collect_expr_names(expr, names);
            }
        }
    }
}

fn collect_provides_names(
    provides: &[daglang_syntax::ast::ProvidesClause],
    names: &mut HashSet<String>,
) {
    for p in provides {
        collect_type_names(&p.resource_type, names);
    }
}

fn collect_capability_names(cap: &CapabilityDef, names: &mut HashSet<String>) {
    collect_fields_names(&cap.inputs, names);
    collect_fields_names(&cap.outputs, names);
}

fn collect_operation_names(op: &OperationDef, names: &mut HashSet<String>) {
    collect_fields_names(&op.inputs, names);
    collect_fields_names(&op.outputs, names);
    if let Some(transport) = &op.transport {
        collect_transport_names(transport, names);
    }
    for entry in &op.response {
        collect_type_names(&entry.response_type, names);
    }
    for entry in &op.exit {
        collect_type_names(&entry.output_type, names);
    }
    for mock in &op.mock_responses {
        collect_expr_names(&mock.body, names);
    }
}

fn collect_transport_names(transport: &TransportBinding, names: &mut HashSet<String>) {
    match transport {
        TransportBinding::Rest {
            body, headers, ..
        } => {
            if let Some(expr) = body {
                collect_expr_names(expr, names);
            }
            if let Some(expr) = headers {
                collect_expr_names(expr, names);
            }
        }
        TransportBinding::Shell { argv } => {
            for expr in argv {
                collect_expr_names(expr, names);
            }
        }
        TransportBinding::File { .. } | TransportBinding::Local => {}
    }
}

fn collect_expect_names(expect: &daglang_syntax::ast::ExpectStmt, names: &mut HashSet<String>) {
    use daglang_syntax::ast::ExpectStmt;
    match expect {
        ExpectStmt::Eq(_, expr)
        | ExpectStmt::Ne(_, expr)
        | ExpectStmt::Lt(_, expr)
        | ExpectStmt::Gt(_, expr)
        | ExpectStmt::Le(_, expr)
        | ExpectStmt::Ge(_, expr)
        | ExpectStmt::Contains(_, expr) => {
            collect_expr_names(expr, names);
        }
        ExpectStmt::Is(_, type_name) => {
            names.insert(type_name.clone());
        }
        ExpectStmt::Truthy(_) => {}
    }
}

fn collect_stmts_names(stmts: &[Stmt], names: &mut HashSet<String>) {
    for stmt in stmts {
        match stmt {
            Stmt::Let(_, expr) | Stmt::Assign(_, expr) | Stmt::Expr(expr) => {
                collect_expr_names(expr, names);
            }
            Stmt::Node(ns) => {
                collect_expr_names(&ns.expr, names);
                if let Some(guard) = &ns.when_guard {
                    collect_expr_names(guard, names);
                }
            }
            Stmt::Return(fields) => {
                for (_, expr) in fields {
                    collect_expr_names(expr, names);
                }
            }
        }
    }
}

fn collect_expr_names(expr: &Expr, names: &mut HashSet<String>) {
    match expr {
        Expr::Ident(name) => {
            names.insert(name.clone());
        }
        Expr::Call(name, args) => {
            names.insert(name.clone());
            for (_, arg) in args {
                collect_expr_names(arg, names);
            }
        }
        Expr::ServiceCall(path, args) => {
            // Insert all dotted prefixes of the service call path so that
            // dotted import bindings like "shell.Codegen" match against
            // calls like shell.Codegen.Check().
            let mut prefix = String::new();
            for (i, segment) in path.iter().enumerate() {
                if i > 0 {
                    prefix.push('.');
                }
                prefix.push_str(segment);
                names.insert(prefix.clone());
            }
            for (_, arg) in args {
                collect_expr_names(arg, names);
            }
        }
        Expr::Record(type_name, fields) => {
            if let Some(name) = type_name {
                names.insert(name.clone());
            }
            for (_, value) in fields {
                collect_expr_names(value, names);
            }
        }
        Expr::FieldAccess(base, _) => collect_expr_names(base, names),
        Expr::BinOp(lhs, _, rhs) => {
            collect_expr_names(lhs, names);
            collect_expr_names(rhs, names);
        }
        Expr::UnaryOp(_, inner) | Expr::Lambda(_, inner) | Expr::After(inner, _) => {
            collect_expr_names(inner, names);
        }
        Expr::For(_, iterable, _, body) => {
            collect_expr_names(iterable, names);
            match body {
                ForBody::Expr(expr) => collect_expr_names(expr, names),
                ForBody::Block(stmts) => collect_stmts_names(stmts, names),
            }
        }
        Expr::StringInterp(parts) => {
            for part in parts {
                if let daglang_syntax::ast::StringPart::Expr(inner) = part {
                    collect_expr_names(inner, names);
                }
            }
        }
        Expr::Return(fields) => {
            for (_, value) in fields {
                collect_expr_names(value, names);
            }
        }
        Expr::Match(scrutinee, arms) => {
            collect_expr_names(scrutinee, names);
            for arm in arms {
                collect_match_arm_names(arm, names);
            }
        }
        Expr::If(cond, then_expr, else_expr) => {
            collect_expr_names(cond, names);
            collect_expr_names(then_expr, names);
            if let Some(otherwise) = else_expr {
                collect_expr_names(otherwise, names);
            }
        }
        Expr::List(items) => {
            for item in items {
                collect_expr_names(item, names);
            }
        }
        Expr::Map(entries) => {
            for (key, value) in entries {
                collect_expr_names(key, names);
                collect_expr_names(value, names);
            }
        }
        Expr::Guarded(inner, guard) => {
            collect_expr_names(inner, names);
            collect_expr_names(guard, names);
        }
        Expr::Block(stmts) => collect_stmts_names(stmts, names),
        Expr::Literal(_) => {}
    }
}

fn collect_match_arm_names(arm: &MatchArm, names: &mut HashSet<String>) {
    collect_pattern_names(&arm.pattern, names);
    if let Some(guard) = &arm.guard {
        collect_expr_names(guard, names);
    }
    collect_expr_names(&arm.body, names);
}

fn collect_pattern_names(pattern: &Pattern, names: &mut HashSet<String>) {
    match pattern {
        Pattern::Variant(name, fields) => {
            names.insert(name.clone());
            for (_, pat) in fields {
                collect_pattern_names(pat, names);
            }
        }
        Pattern::Ident(_) | Pattern::Wildcard | Pattern::Literal(_) => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use daglang_syntax::parser;

    fn parse(source: &str) -> SourceFile {
        parser::parse(source).expect("test source should parse")
    }

    #[test]
    fn binding_used_as_type_is_not_reported() {
        let source = parse(
            r#"
            module test.a
            import std.types { Summary }
            func greet() -> { result: Summary } {
                return { result: "ok" }
            }
            "#,
        );
        assert!(find_unused_imports(&source).is_empty());
    }

    #[test]
    fn binding_used_as_call_is_not_reported() {
        let source = parse(
            r#"
            module test.a
            import shared.util { format_report }
            func greet() -> { result: String } {
                node r = format_report(text: "ok")
                return { result: r }
            }
            "#,
        );
        assert!(find_unused_imports(&source).is_empty());
    }

    #[test]
    fn unused_binding_is_reported() {
        let source = parse(
            r#"
            module test.a
            import std.types { Summary, Unused }
            func greet() -> { result: Summary } {
                return { result: "ok" }
            }
            "#,
        );
        let unused = find_unused_imports(&source);
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].binding.as_deref(), Some("Unused"));
        assert_eq!(unused[0].module_path, "std.types");
    }

    #[test]
    fn module_import_without_export_index_is_skipped() {
        let source = parse(
            r#"
            module test.a
            import extdeps.cargo
            service cargo.Build {
                operation Build() -> { success: Bool, stdout: String, stderr: String } {
                    transport shell { argv: ["cargo", "build"] }
                }
            }
            "#,
        );
        // Without an export index, non-aliased module-level imports cannot be
        // verified structurally and are conservatively treated as used.
        let unused = find_unused_imports(&source);
        assert!(
            unused.is_empty(),
            "non-aliased module imports without export index should be skipped, got: {unused:?}"
        );
    }

    #[test]
    fn module_import_unused_with_export_index_is_reported() {
        let source = parse(
            r#"
            module test.a
            import extdeps.cargo
            service cargo.Build {
                operation Build() -> { success: Bool, stdout: String, stderr: String } {
                    transport shell { argv: ["cargo", "build"] }
                }
            }
            "#,
        );
        // The service definition does not reference "cargo" as an expression;
        // service names are string literals. With an export index showing what
        // extdeps.cargo actually exports, the import is correctly unused.
        let mut export_index = ModuleExportIndex::new();
        export_index.insert(
            "extdeps.cargo".into(),
            ["cargo.Build".into()].into_iter().collect(),
        );
        let unused = find_unused_imports_with_export_index(&source, &export_index);
        assert_eq!(unused.len(), 1);
        assert_eq!(unused[0].module_path, "extdeps.cargo");
        assert!(unused[0].binding.is_none());
    }

    #[test]
    fn module_import_used_via_export_index_is_not_reported() {
        let source = parse(
            r#"
            module test.a
            import extdeps.cloud.gcp.sts
            func auth() -> { ok: Bool } {
                token = gcp.STS.Exchange()
                return { ok: true }
            }
            "#,
        );
        let mut export_index = ModuleExportIndex::new();
        export_index.insert(
            "extdeps.cloud.gcp.sts".into(),
            ["gcp.STS".into()].into_iter().collect(),
        );
        let unused = find_unused_imports_with_export_index(&source, &export_index);
        assert!(
            unused.is_empty(),
            "import used via exported namespace should not be reported, got: {unused:?}"
        );
    }

    #[test]
    fn all_bindings_used_returns_empty() {
        let source = parse(
            r#"
            module test.a
            import std.types { FilePath, Milliseconds }
            fn example(path: FilePath, timeout: Milliseconds) -> String {
                path
            }
            "#,
        );
        assert!(find_unused_imports(&source).is_empty());
    }

    #[test]
    fn binding_used_in_uses_clause_is_not_reported() {
        let source = parse(
            r#"
            module test.a
            import std.resources { Filesystem }
            func write_file() -> { ok: Bool }
                uses fs: Filesystem
            {
                return { ok: true }
            }
            "#,
        );
        assert!(find_unused_imports(&source).is_empty());
    }

    #[test]
    fn binding_used_in_type_alias_is_not_reported() {
        let source = parse(
            r#"
            module test.a
            import std.types { FilePath }
            type MyPath = FilePath
            "#,
        );
        assert!(find_unused_imports(&source).is_empty());
    }

    #[test]
    fn binding_used_in_record_field_is_not_reported() {
        let source = parse(
            r#"
            module test.a
            import std.types { FilePath }
            type Config {
                path: FilePath
            }
            "#,
        );
        assert!(find_unused_imports(&source).is_empty());
    }

    #[test]
    fn binding_used_in_generic_type_is_not_reported() {
        let source = parse(
            r#"
            module test.a
            import std.types { FilePath }
            fn paths() -> List<FilePath> {
                []
            }
            "#,
        );
        assert!(find_unused_imports(&source).is_empty());
    }

    #[test]
    fn binding_used_in_pattern_match_is_not_reported() {
        let source = parse(
            r#"
            module test.a
            import std.types { Success }
            fn check(r: Result) -> String {
                match r {
                    Success { value } => value
                    _ => "error"
                }
            }
            "#,
        );
        assert!(find_unused_imports(&source).is_empty());
    }

    #[test]
    fn no_imports_returns_empty() {
        let source = parse(
            r#"
            module test.a
            fn greet() -> String { "hello" }
            "#,
        );
        assert!(find_unused_imports(&source).is_empty());
    }

    #[test]
    fn binding_used_in_uses_clause_with_config_is_not_reported() {
        let source = parse(
            r#"
            module test.a
            import std.resources { Filesystem }
            func write_file() -> { ok: Bool }
                uses fs: Filesystem(mode: ReadWrite)
            {
                return { ok: true }
            }
            "#,
        );
        let unused = find_unused_imports(&source);
        assert!(
            unused.is_empty(),
            "expected no unused imports but got: {unused:?}"
        );
    }

    #[test]
    fn multiple_unused_bindings_all_reported() {
        let source = parse(
            r#"
            module test.a
            import std.types { A, B, C }
            fn greet() -> String { "hello" }
            "#,
        );
        let unused = find_unused_imports(&source);
        assert_eq!(unused.len(), 3);
        let binding_names: Vec<_> = unused.iter().filter_map(|u| u.binding.as_deref()).collect();
        assert!(binding_names.contains(&"A"));
        assert!(binding_names.contains(&"B"));
        assert!(binding_names.contains(&"C"));
    }
}
