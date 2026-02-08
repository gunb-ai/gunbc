#![deny(dead_code)]
use proc_macro::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::{parse_macro_input, AttributeArgs, Expr, ItemFn, Lit, Meta, NestedMeta};

#[proc_macro_attribute]
pub fn testgen_target(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut name: Option<syn::LitStr> = None;
    let mut output: Option<syn::LitStr> = None;
    let mut module: Option<syn::LitStr> = None;
    let mut builder: Option<Expr> = None;
    let mut signature: Option<Expr> = None;
    let mut flow_tests = false;
    let mut live_flow_tests = false;
    let mut no_boundary_tests = false;
    let mut no_chain_tests = false;
    let mut skip = false;
    let mut window_max_nodes: Option<usize> = None;
    let mut test_class: Option<syn::LitStr> = None;
    let mut fermi_cost: Option<syn::LitStr> = None;
    let mut requires: Option<Vec<syn::LitStr>> = None;
    let mut secrets: Option<Vec<syn::LitStr>> = None;
    let mut live_test_class: Option<syn::LitStr> = None;
    let mut live_fermi_cost: Option<syn::LitStr> = None;
    let mut live_requires: Option<Vec<syn::LitStr>> = None;
    let mut live_required: Option<Vec<syn::LitStr>> = None;
    let mut live_required_any_of: Vec<Vec<syn::LitStr>> = Vec::new();
    let mut tool: Option<syn::LitStr> = None;

    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                let ident = nv.path.get_ident().map(|i| i.to_string());
                match ident.as_deref() {
                    Some("name") => {
                        if let Lit::Str(s) = nv.lit { name = Some(s); }
                        else { return syn::Error::new_spanned(nv, "name must be a string literal").to_compile_error().into(); }
                    }
                    Some("output") => {
                        if let Lit::Str(s) = nv.lit { output = Some(s); }
                        else { return syn::Error::new_spanned(nv, "output must be a string literal").to_compile_error().into(); }
                    }
                    Some("module") => {
                        if let Lit::Str(s) = nv.lit { module = Some(s); }
                        else { return syn::Error::new_spanned(nv, "module must be a string literal").to_compile_error().into(); }
                    }
                    Some("builder") => {
                        if let Lit::Str(s) = nv.lit {
                            match syn::parse_str::<Expr>(&s.value()) {
                                Ok(expr) => builder = Some(expr),
                                Err(e) => return e.to_compile_error().into(),
                            }
                        } else {
                            return syn::Error::new_spanned(nv, "builder must be a string literal or use builder(...) form").to_compile_error().into();
                        }
                    }
                    Some("signature") => {
                        if let Lit::Str(s) = nv.lit {
                            match syn::parse_str::<Expr>(&s.value()) {
                                Ok(expr) => signature = Some(expr),
                                Err(e) => return e.to_compile_error().into(),
                            }
                        } else {
                            return syn::Error::new_spanned(nv, "signature must be a string literal or use signature(...) form").to_compile_error().into();
                        }
                    }
                    Some("window_max_nodes") => {
                        if let Lit::Int(i) = nv.lit {
                            match i.base10_parse::<usize>() {
                                Ok(v) => window_max_nodes = Some(v),
                                Err(e) => return e.to_compile_error().into(),
                            }
                        } else {
                            return syn::Error::new_spanned(nv, "window_max_nodes must be an integer").to_compile_error().into();
                        }
                    }
                    Some("class") => {
                        if let Lit::Str(s) = nv.lit { test_class = Some(s); }
                        else { return syn::Error::new_spanned(nv, "class must be a string literal").to_compile_error().into(); }
                    }
                    Some("fermi") => {
                        if let Lit::Str(s) = nv.lit { fermi_cost = Some(s); }
                        else { return syn::Error::new_spanned(nv, "fermi must be a string literal").to_compile_error().into(); }
                    }
                    Some("live_class") => {
                        if let Lit::Str(s) = nv.lit { live_test_class = Some(s); }
                        else { return syn::Error::new_spanned(nv, "live_class must be a string literal").to_compile_error().into(); }
                    }
                    Some("live_fermi") => {
                        if let Lit::Str(s) = nv.lit { live_fermi_cost = Some(s); }
                        else { return syn::Error::new_spanned(nv, "live_fermi must be a string literal").to_compile_error().into(); }
                    }
                    Some("tool") => {
                        if let Lit::Str(s) = nv.lit { tool = Some(s); }
                        else { return syn::Error::new_spanned(nv, "tool must be a string literal").to_compile_error().into(); }
                    }
                    _ => {
                        return syn::Error::new_spanned(nv, "unknown testgen_target argument").to_compile_error().into();
                    }
                }
            }
            NestedMeta::Meta(Meta::List(list)) => {
                let ident = list.path.get_ident().map(|i| i.to_string());
                match ident.as_deref() {
                    Some("builder") => {
                        match syn::parse2::<Expr>(list.nested.to_token_stream()) {
                            Ok(expr) => builder = Some(expr),
                            Err(e) => return e.to_compile_error().into(),
                        }
                    }
                    Some("signature") => {
                        match syn::parse2::<Expr>(list.nested.to_token_stream()) {
                            Ok(expr) => signature = Some(expr),
                            Err(e) => return e.to_compile_error().into(),
                        }
                    }
                    Some("requires") => {
                        let mut items = Vec::new();
                        for item in list.nested.iter() {
                            match item {
                                NestedMeta::Lit(Lit::Str(s)) => items.push(s.clone()),
                                _ => {
                                    return syn::Error::new_spanned(
                                        item,
                                        "requires(...) only supports string literals",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            }
                        }
                        requires = Some(items);
                    }
                    Some("secrets") => {
                        let mut items = Vec::new();
                        for item in list.nested.iter() {
                            match item {
                                NestedMeta::Lit(Lit::Str(s)) => items.push(s.clone()),
                                _ => {
                                    return syn::Error::new_spanned(
                                        item,
                                        "secrets(...) only supports string literals",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            }
                        }
                        secrets = Some(items);
                    }
                    Some("live_requires") => {
                        let mut items = Vec::new();
                        for item in list.nested.iter() {
                            match item {
                                NestedMeta::Lit(Lit::Str(s)) => items.push(s.clone()),
                                _ => {
                                    return syn::Error::new_spanned(
                                        item,
                                        "live_requires(...) only supports string literals",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            }
                        }
                        live_requires = Some(items);
                    }
                    Some("live_required") => {
                        let mut items = Vec::new();
                        for item in list.nested.iter() {
                            match item {
                                NestedMeta::Lit(Lit::Str(s)) => items.push(s.clone()),
                                _ => {
                                    return syn::Error::new_spanned(
                                        item,
                                        "live_required(...) only supports string literals",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            }
                        }
                        live_required = Some(items);
                    }
                    Some("live_required_any_of") => {
                        let mut items = Vec::new();
                        for item in list.nested.iter() {
                            match item {
                                NestedMeta::Lit(Lit::Str(s)) => items.push(s.clone()),
                                _ => {
                                    return syn::Error::new_spanned(
                                        item,
                                        "live_required_any_of(...) only supports string literals",
                                    )
                                    .to_compile_error()
                                    .into();
                                }
                            }
                        }
                        live_required_any_of.push(items);
                    }
                    _ => {
                        return syn::Error::new_spanned(list, "unknown testgen_target list argument").to_compile_error().into();
                    }
                }
            }
            NestedMeta::Meta(Meta::Path(path)) => {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "flow_tests" => flow_tests = true,
                        "live_flow_tests" => live_flow_tests = true,
                        "no_boundary_tests" => no_boundary_tests = true,
                        "no_chain_tests" => no_chain_tests = true,
                        "skip" => skip = true,
                        _ => {
                            return syn::Error::new_spanned(path, "unknown testgen_target flag").to_compile_error().into();
                        }
                    }
                }
            }
            _ => {
                return syn::Error::new_spanned(arg, "unsupported testgen_target argument").to_compile_error().into();
            }
        }
    }

    if skip {
        if name.is_some()
            || output.is_some()
            || module.is_some()
            || signature.is_some()
            || flow_tests
            || live_flow_tests
            || no_boundary_tests
            || no_chain_tests
            || window_max_nodes.is_some()
            || test_class.is_some()
            || fermi_cost.is_some()
            || requires.is_some()
            || secrets.is_some()
            || live_test_class.is_some()
            || live_fermi_cost.is_some()
            || live_requires.is_some()
            || live_required.is_some()
            || !live_required_any_of.is_empty()
            || tool.is_some()
        {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "testgen_target(skip) cannot be combined with other arguments (except builder for coverage tracking)",
            )
            .to_compile_error()
            .into();
        }
        // skip emits the function unchanged — builder is allowed purely so the
        // source-level coverage scanner in tool_registration.rs can see it.
        return quote!(#input_fn).into();
    }

    let name = match name { Some(v) => v, None => return syn::Error::new_spanned(&input_fn.sig.ident, "testgen_target requires name = \"...\"").to_compile_error().into() };
    let output = match output { Some(v) => v, None => return syn::Error::new_spanned(&input_fn.sig.ident, "testgen_target requires output = \"...\"").to_compile_error().into() };
    let module = match module { Some(v) => v, None => return syn::Error::new_spanned(&input_fn.sig.ident, "testgen_target requires module = \"...\"").to_compile_error().into() };
    let builder = match builder { Some(v) => v, None => return syn::Error::new_spanned(&input_fn.sig.ident, "testgen_target requires builder(...) or builder = \"...\"").to_compile_error().into() };

    let fn_ident = input_fn.sig.ident.clone();
    let gen_ident = format_ident!("__testgen_generate_{}", fn_ident);

    let mut boundary_tests = true;
    let mut chain_tests = true;
    let mut flow = false;

    if no_boundary_tests {
        boundary_tests = false;
    }
    if no_chain_tests {
        chain_tests = false;
    }
    if flow_tests {
        flow = true;
        boundary_tests = false;
        chain_tests = false;
    }

    let signature_tokens = if let Some(sig) = signature {
        quote!(Some(stringify!(#sig)))
    } else {
        quote!(None)
    };

    let window_tokens = if let Some(n) = window_max_nodes {
        quote!(Some(#n))
    } else {
        quote!(None)
    };

    let class_tokens = if let Some(class) = test_class {
        match class.value().to_lowercase().as_str() {
            "unit" => quote!(Some(gunbc_test::TestClass::Unit)),
            "hermetic" => quote!(Some(gunbc_test::TestClass::Hermetic)),
            "integration" => quote!(Some(gunbc_test::TestClass::Integration)),
            _ => {
                return syn::Error::new_spanned(
                    class,
                    "class must be one of: unit, hermetic, integration",
                )
                .to_compile_error()
                .into();
            }
        }
    } else {
        quote!(None)
    };

    let fermi_tokens = if let Some(cost) = fermi_cost {
        match cost.value().to_uppercase().as_str() {
            "XS" => quote!(Some(gunbc_test::FermiCost::XS)),
            "S" => quote!(Some(gunbc_test::FermiCost::S)),
            "M" => quote!(Some(gunbc_test::FermiCost::M)),
            "L" => quote!(Some(gunbc_test::FermiCost::L)),
            "XL" => quote!(Some(gunbc_test::FermiCost::XL)),
            _ => {
                return syn::Error::new_spanned(
                    cost,
                    "fermi must be one of: XS, S, M, L, XL",
                )
                .to_compile_error()
                .into();
            }
        }
    } else {
        quote!(None)
    };

    let requires_tokens = if let Some(list) = requires {
        quote!(Some(&[#(#list),*]))
    } else {
        quote!(None)
    };

    let secrets_tokens = if let Some(list) = secrets {
        quote!(Some(&[#(#list),*]))
    } else {
        quote!(None)
    };

    let live_class_tokens = if let Some(class) = live_test_class {
        match class.value().to_lowercase().as_str() {
            "unit" => quote!(Some(gunbc_test::TestClass::Unit)),
            "hermetic" => quote!(Some(gunbc_test::TestClass::Hermetic)),
            "integration" => quote!(Some(gunbc_test::TestClass::Integration)),
            _ => {
                return syn::Error::new_spanned(
                    class,
                    "live_class must be one of: unit, hermetic, integration",
                )
                .to_compile_error()
                .into();
            }
        }
    } else {
        quote!(None)
    };

    let live_fermi_tokens = if let Some(cost) = live_fermi_cost {
        match cost.value().to_uppercase().as_str() {
            "XS" => quote!(Some(gunbc_test::FermiCost::XS)),
            "S" => quote!(Some(gunbc_test::FermiCost::S)),
            "M" => quote!(Some(gunbc_test::FermiCost::M)),
            "L" => quote!(Some(gunbc_test::FermiCost::L)),
            "XL" => quote!(Some(gunbc_test::FermiCost::XL)),
            _ => {
                return syn::Error::new_spanned(
                    cost,
                    "live_fermi must be one of: XS, S, M, L, XL",
                )
                .to_compile_error()
                .into();
            }
        }
    } else {
        quote!(None)
    };

    let live_requires_tokens = if let Some(list) = live_requires {
        quote!(Some(&[#(#list),*]))
    } else {
        quote!(None)
    };

    let live_required_tokens = if let Some(list) = live_required {
        quote!(Some(&[#(#list),*]))
    } else {
        quote!(None)
    };

    let live_required_any_of_tokens = if !live_required_any_of.is_empty() {
        let groups: Vec<_> = live_required_any_of
            .iter()
            .map(|group| quote!(&[#(#group),*]))
            .collect();
        quote!(Some(&[#(#groups),*]))
    } else {
        quote!(None)
    };

    let tool_tokens = if let Some(t) = tool {
        quote!(Some(#t))
    } else {
        quote!(None)
    };

    let expanded = quote! {
        #input_fn

        fn #gen_ident(config: &gunbc_testgen_registry::TestgenTargetDef) -> String {
            let dag = #builder;
            let spec = #fn_ident();
            gunbc_testgen_registry::generate_target(config, dag, spec)
        }

        gunbc_testgen_registry::inventory::submit! {
            gunbc_testgen_registry::DagSpecDef {
                origin_crate: env!("CARGO_CRATE_NAME"),
                name: #name,
                dag_builder_call: stringify!(#builder),
                mock_spec_path: concat!(module_path!(), "::", stringify!(#fn_ident), "()"),
                signature_path: #signature_tokens,
                meta: gunbc_testgen_registry::DagSpecMeta {
                    output_path: #output,
                    module_name: #module,
                    tool_name: #tool_tokens,
                },
                testgen: gunbc_testgen_registry::DagSpecTestgen {
                    boundary_tests: #boundary_tests,
                    chain_tests: #chain_tests,
                    flow_tests: #flow,
                    live_flow_tests: #live_flow_tests,
                    window_max_nodes: #window_tokens,
                    test_class: #class_tokens,
                    fermi_cost: #fermi_tokens,
                    requires: #requires_tokens,
                    secrets: #secrets_tokens,
                    live_test_class: #live_class_tokens,
                    live_fermi_cost: #live_fermi_tokens,
                    live_requires: #live_requires_tokens,
                    live_required: #live_required_tokens,
                    live_required_any_of: #live_required_any_of_tokens,
                },
                generate: #gen_ident,
            }
        }
    };

    expanded.into()
}

#[proc_macro_attribute]
pub fn resource_test_target(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut name: Option<syn::LitStr> = None;
    let mut builder: Option<Expr> = None;
    let mut skip = false;

    for arg in args {
        match arg {
            NestedMeta::Meta(Meta::NameValue(nv)) => {
                let ident = nv.path.get_ident().map(|i| i.to_string());
                match ident.as_deref() {
                    Some("name") => {
                        if let Lit::Str(s) = nv.lit {
                            name = Some(s);
                        } else {
                            return syn::Error::new_spanned(nv, "name must be a string literal")
                                .to_compile_error()
                                .into();
                        }
                    }
                    Some("builder") => {
                        if let Lit::Str(s) = nv.lit {
                            match syn::parse_str::<Expr>(&s.value()) {
                                Ok(expr) => builder = Some(expr),
                                Err(e) => return e.to_compile_error().into(),
                            }
                        } else {
                            return syn::Error::new_spanned(
                                nv,
                                "builder must be a string literal or use builder(...) form",
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                    _ => {
                        return syn::Error::new_spanned(nv, "unknown resource_test_target argument")
                            .to_compile_error()
                            .into();
                    }
                }
            }
            NestedMeta::Meta(Meta::List(list)) => {
                let ident = list.path.get_ident().map(|i| i.to_string());
                match ident.as_deref() {
                    Some("builder") => match syn::parse2::<Expr>(list.nested.to_token_stream()) {
                        Ok(expr) => builder = Some(expr),
                        Err(e) => return e.to_compile_error().into(),
                    },
                    _ => {
                        return syn::Error::new_spanned(list, "unknown resource_test_target list argument")
                            .to_compile_error()
                            .into();
                    }
                }
            }
            NestedMeta::Meta(Meta::Path(path)) => {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "skip" => skip = true,
                        _ => {
                            return syn::Error::new_spanned(path, "unknown resource_test_target flag")
                                .to_compile_error()
                                .into();
                        }
                    }
                }
            }
            _ => {
                return syn::Error::new_spanned(arg, "unsupported resource_test_target argument")
                    .to_compile_error()
                    .into();
            }
        }
    }

    if skip {
        return quote!(#input_fn).into();
    }

    let name = match name {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "resource_test_target requires name = \"...\"",
            )
            .to_compile_error()
            .into()
        }
    };
    let builder = match builder {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "resource_test_target requires builder(...) or builder = \"...\"",
            )
            .to_compile_error()
            .into()
        }
    };

    let fn_ident = input_fn.sig.ident.clone();
    let gen_ident = format_ident!("__resource_test_build_{}", fn_ident);

    let expanded = quote! {
        #input_fn

        fn #gen_ident() -> gunbc_ir::Dag<()> {
            let dag = #builder;
            let mut mapper = |_| ();
            dag.map_ops(&mut mapper)
        }

        gunbc_testgen_registry::inventory::submit! {
            gunbc_testgen_registry::ResourceTestDef {
                origin_crate: env!("CARGO_CRATE_NAME"),
                name: #name,
                build: #gen_ident,
            }
        }
    };

    expanded.into()
}
