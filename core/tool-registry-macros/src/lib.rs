#![deny(dead_code)]
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, AttributeArgs, ItemFn, Lit, Meta, NestedMeta};

/// Register a tool target for auto-discovery.
///
/// # Required arguments
///
/// - `name = "..."` — Tool name for CLI and display (e.g., "gist")
/// - `crate_name = "..."` — Cargo crate name (e.g., "gunbc-gist")
/// - `description = "..."` — Short description
/// - `builder = "..."` — Graph builder call expression
///
/// # Optional arguments
///
/// - `args = "..."` — Arguments to pass to graph builder
/// - `import = "..."` — Custom import line
/// - `success_port = "..."` — Output port to check for success
/// - `returns_result` — Graph builder returns `Result<Dag, BuilderError>`
/// - `enable_step_mode` — Generate step subcommand for CI
/// - `skip` — Skip registration (emit function only, no inventory submit)
///
/// # Example
///
/// ```ignore
/// #[tool_target(
///     name = "gist",
///     crate_name = "gunbc-gist",
///     description = "Create a GitHub gist from code files",
///     builder = "build_gist_graph(GistMode::Snapshot, extensions.clone(), public)",
///     returns_result
/// )]
/// pub fn gist_tool() {}
/// ```
#[proc_macro_attribute]
pub fn tool_target(args: TokenStream, input: TokenStream) -> TokenStream {
    let args = parse_macro_input!(args as AttributeArgs);
    let input_fn = parse_macro_input!(input as ItemFn);

    let mut name: Option<syn::LitStr> = None;
    let mut crate_name: Option<syn::LitStr> = None;
    let mut description: Option<syn::LitStr> = None;
    let mut builder: Option<syn::LitStr> = None;
    let mut builder_args: Option<syn::LitStr> = None;
    let mut custom_import: Option<syn::LitStr> = None;
    let mut success_port: Option<syn::LitStr> = None;
    let mut returns_result = false;
    let mut enable_step_mode = false;
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
                    Some("crate_name") => {
                        if let Lit::Str(s) = nv.lit {
                            crate_name = Some(s);
                        } else {
                            return syn::Error::new_spanned(
                                nv,
                                "crate_name must be a string literal",
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                    Some("description") => {
                        if let Lit::Str(s) = nv.lit {
                            description = Some(s);
                        } else {
                            return syn::Error::new_spanned(
                                nv,
                                "description must be a string literal",
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                    Some("builder") => {
                        if let Lit::Str(s) = nv.lit {
                            builder = Some(s);
                        } else {
                            return syn::Error::new_spanned(
                                nv,
                                "builder must be a string literal",
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                    Some("args") => {
                        if let Lit::Str(s) = nv.lit {
                            builder_args = Some(s);
                        } else {
                            return syn::Error::new_spanned(nv, "args must be a string literal")
                                .to_compile_error()
                                .into();
                        }
                    }
                    Some("import") => {
                        if let Lit::Str(s) = nv.lit {
                            custom_import = Some(s);
                        } else {
                            return syn::Error::new_spanned(nv, "import must be a string literal")
                                .to_compile_error()
                                .into();
                        }
                    }
                    Some("success_port") => {
                        if let Lit::Str(s) = nv.lit {
                            success_port = Some(s);
                        } else {
                            return syn::Error::new_spanned(
                                nv,
                                "success_port must be a string literal",
                            )
                            .to_compile_error()
                            .into();
                        }
                    }
                    _ => {
                        return syn::Error::new_spanned(nv, "unknown tool_target argument")
                            .to_compile_error()
                            .into();
                    }
                }
            }
            NestedMeta::Meta(Meta::Path(path)) => {
                if let Some(ident) = path.get_ident() {
                    match ident.to_string().as_str() {
                        "returns_result" => returns_result = true,
                        "enable_step_mode" => enable_step_mode = true,
                        "skip" => skip = true,
                        _ => {
                            return syn::Error::new_spanned(path, "unknown tool_target flag")
                                .to_compile_error()
                                .into();
                        }
                    }
                }
            }
            _ => {
                return syn::Error::new_spanned(arg, "unsupported tool_target argument")
                    .to_compile_error()
                    .into();
            }
        }
    }

    // skip mode: emit function only, no inventory registration
    if skip {
        if name.is_some()
            || crate_name.is_some()
            || description.is_some()
            || builder.is_some()
            || builder_args.is_some()
            || custom_import.is_some()
            || success_port.is_some()
            || returns_result
            || enable_step_mode
        {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "tool_target(skip) cannot be combined with other arguments",
            )
            .to_compile_error()
            .into();
        }
        return quote!(#input_fn).into();
    }

    // Validate required fields
    let name = match name {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "tool_target requires name = \"...\"",
            )
            .to_compile_error()
            .into()
        }
    };
    let crate_name = match crate_name {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "tool_target requires crate_name = \"...\"",
            )
            .to_compile_error()
            .into()
        }
    };
    let description = match description {
        Some(v) => v,
        None => {
            return syn::Error::new_spanned(
                &input_fn.sig.ident,
                "tool_target requires description = \"...\"",
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
                "tool_target requires builder = \"...\"",
            )
            .to_compile_error()
            .into()
        }
    };

    let args_tokens = match builder_args {
        Some(s) => quote!(#s),
        None => quote!(""),
    };

    let import_tokens = match custom_import {
        Some(s) => quote!(Some(#s)),
        None => quote!(None),
    };

    let success_port_tokens = match success_port {
        Some(s) => quote!(Some(#s)),
        None => quote!(None),
    };

    let expanded = quote! {
        #input_fn

        gunbc_tool_registry::inventory::submit! {
            gunbc_tool_registry::ToolRegistration {
                origin_crate: env!("CARGO_CRATE_NAME"),
                crate_name: #crate_name,
                tool_name: #name,
                description: #description,
                graph_builder_call: #builder,
                graph_builder_args: #args_tokens,
                returns_result: #returns_result,
                success_port: #success_port_tokens,
                enable_step_mode: #enable_step_mode,
                custom_import: #import_tokens,
            }
        }
    };

    expanded.into()
}
