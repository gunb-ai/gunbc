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
    let mut no_boundary_tests = false;
    let mut no_chain_tests = false;
    let mut skip = false;
    let mut window_max_nodes: Option<usize> = None;

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
                    _ => {
                        return syn::Error::new_spanned(list, "unknown testgen_target list argument").to_compile_error().into();
                    }
                }
            }
            NestedMeta::Meta(Meta::Path(path)) => {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "flow_tests" => flow_tests = true,
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
            || builder.is_some()
            || signature.is_some()
            || flow_tests
            || no_boundary_tests
            || no_chain_tests
            || window_max_nodes.is_some()
        {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "testgen_target(skip) cannot be combined with other arguments",
            )
            .to_compile_error()
            .into();
        }
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

    let expanded = quote! {
        #input_fn

        fn #gen_ident(config: &gunbc_testgen_registry::TestgenTargetDef) -> String {
            let dag = #builder;
            let spec = #fn_ident();
            gunbc_testgen_registry::generate_target(config, dag, spec)
        }

        gunbc_testgen_registry::inventory::submit! {
            gunbc_testgen_registry::TestgenTarget {
                origin_crate: env!("CARGO_CRATE_NAME"),
                name: #name,
                output_path: #output,
                module_name: #module,
                dag_builder_call: stringify!(#builder),
                mock_spec_path: concat!(module_path!(), "::", stringify!(#fn_ident), "()"),
                signature_path: #signature_tokens,
                boundary_tests: #boundary_tests,
                chain_tests: #chain_tests,
                flow_tests: #flow,
                window_max_nodes: #window_tokens,
                generate: #gen_ident,
            }
        }
    };

    expanded.into()
}
