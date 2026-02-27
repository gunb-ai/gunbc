use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, DeriveInput, Field, Fields};

#[proc_macro_derive(DelegateExecutable)]
pub fn derive_delegate_executable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = input.ident;

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new_spanned(
            enum_ident,
            "DelegateExecutable can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    let mut arms = Vec::new();
    for variant in data_enum.variants {
        let variant_ident = variant.ident;
        match extract_single_field_pattern(&variant.fields) {
            Ok(pattern) => arms.push(quote! {
                Self::#variant_ident #pattern => inner.execute(inputs),
            }),
            Err(err) => return err.to_compile_error().into(),
        }
    }

    quote! {
        impl gunbc_exec::Executable for #enum_ident {
            fn execute(
                &self,
                inputs: std::collections::HashMap<String, gunbc_ir::Value>,
            ) -> Result<
                std::collections::HashMap<String, gunbc_ir::Value>,
                gunbc_exec::ExecError,
            > {
                match self {
                    #(#arms)*
                }
            }
        }
    }
    .into()
}

#[proc_macro_derive(DelegateMockable)]
pub fn derive_delegate_mockable(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = input.ident;

    let Data::Enum(data_enum) = input.data else {
        return syn::Error::new_spanned(
            enum_ident,
            "DelegateMockable can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    let mut mock_arms = Vec::new();
    let mut cardinality_arms = Vec::new();
    let mut error_arms = Vec::new();

    for variant in data_enum.variants {
        let variant_ident = variant.ident;
        match extract_single_field_pattern(&variant.fields) {
            Ok(pattern) => {
                mock_arms.push(quote! {
                    Self::#variant_ident #pattern => inner.mock_outputs(),
                });
                cardinality_arms.push(quote! {
                    Self::#variant_ident #pattern => inner.cardinality_inputs(),
                });
                error_arms.push(quote! {
                    Self::#variant_ident #pattern => inner.error_cases(),
                });
            }
            Err(err) => return err.to_compile_error().into(),
        }
    }

    quote! {
        impl gunbc_test::Mockable for #enum_ident {
            fn mock_outputs(&self) -> std::collections::HashMap<String, gunbc_ir::Value> {
                match self {
                    #(#mock_arms)*
                }
            }

            fn cardinality_inputs(&self) -> Vec<gunbc_test::CardinalityTestInput> {
                match self {
                    #(#cardinality_arms)*
                }
            }

            fn error_cases(&self) -> Vec<gunbc_test::ErrorTestCase> {
                match self {
                    #(#error_arms)*
                }
            }
        }
    }
    .into()
}

fn extract_single_field_pattern(fields: &Fields) -> Result<proc_macro2::TokenStream, syn::Error> {
    match fields {
        Fields::Unnamed(fields_unnamed) if fields_unnamed.unnamed.len() == 1 => Ok(quote!((inner))),
        Fields::Named(fields_named) if fields_named.named.len() == 1 => {
            let field: &Field = fields_named.named.iter().next().expect("one field");
            let field_ident = field.ident.as_ref().expect("named field");
            Ok(quote!({ #field_ident: inner }))
        }
        _ => Err(syn::Error::new_spanned(
            fields,
            "Delegate macros require enum variants with exactly one field",
        )),
    }
}

// =============================================================================
// StringEnum derive macro
// =============================================================================

/// Derive `as_str()`, `parse()`, and `Display` for simple unit-variant enums.
///
/// # Container attributes
///
/// - `#[string_enum(rename_all = "UPPERCASE")]` — convert variant names to uppercase.
///   Default is lowercase.
///
/// # Variant attributes
///
/// - `#[string_enum(name = "custom")]` — override the string for this variant.
///
/// # Generated methods
///
/// - `fn as_str(&self) -> &'static str` — canonical string representation.
/// - `fn parse(s: &str) -> Option<Self>` — case-insensitive parse.
/// - `impl Display` — delegates to `as_str()`.
#[proc_macro_derive(StringEnum, attributes(string_enum))]
pub fn derive_string_enum(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let enum_ident = &input.ident;

    let Data::Enum(ref data_enum) = input.data else {
        return syn::Error::new_spanned(
            enum_ident,
            "StringEnum can only be derived for enums",
        )
        .to_compile_error()
        .into();
    };

    let rename_all = match parse_container_rename_all(&input.attrs) {
        Ok(v) => v,
        Err(e) => return e.to_compile_error().into(),
    };

    let mut as_str_arms = Vec::new();
    let mut parse_arms = Vec::new();

    for variant in &data_enum.variants {
        if !variant.fields.is_empty() {
            return syn::Error::new_spanned(
                &variant.fields,
                "StringEnum requires unit variants (no fields)",
            )
            .to_compile_error()
            .into();
        }

        let ident = &variant.ident;

        let string_value = match parse_variant_string_name(&variant.attrs) {
            Ok(Some(custom)) => custom,
            Ok(None) => apply_rename(&ident.to_string(), rename_all.as_deref()),
            Err(e) => return e.to_compile_error().into(),
        };

        let lower = string_value.to_lowercase();

        as_str_arms.push(quote! {
            Self::#ident => #string_value,
        });
        parse_arms.push(quote! {
            #lower => ::core::option::Option::Some(Self::#ident),
        });
    }

    let expanded = quote! {
        impl #enum_ident {
            /// Get the canonical string representation.
            pub fn as_str(&self) -> &'static str {
                match self {
                    #(#as_str_arms)*
                }
            }

            /// Parse from a case-insensitive string.
            pub fn parse(s: &str) -> ::core::option::Option<Self> {
                match s.to_lowercase().as_str() {
                    #(#parse_arms)*
                    _ => ::core::option::Option::None,
                }
            }
        }

        impl ::core::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                ::core::write!(f, "{}", self.as_str())
            }
        }
    };

    expanded.into()
}

fn parse_container_rename_all(attrs: &[syn::Attribute]) -> Result<Option<String>, syn::Error> {
    for attr in attrs {
        if attr.path().is_ident("string_enum") {
            let mut rename_all = None;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("rename_all") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    let val = lit.value();
                    match val.as_str() {
                        "lowercase" | "UPPERCASE" => {}
                        _ => {
                            return Err(meta.error(format!(
                                "unsupported rename_all value \"{val}\"; \
                                 expected \"lowercase\" or \"UPPERCASE\""
                            )));
                        }
                    }
                    rename_all = Some(val);
                    Ok(())
                } else {
                    Err(meta.error("unrecognized string_enum attribute; expected `rename_all`"))
                }
            })?;
            return Ok(rename_all);
        }
    }
    Ok(None)
}

fn parse_variant_string_name(attrs: &[syn::Attribute]) -> Result<Option<String>, syn::Error> {
    for attr in attrs {
        if attr.path().is_ident("string_enum") {
            let mut name = None;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("name") {
                    let value = meta.value()?;
                    let lit: syn::LitStr = value.parse()?;
                    name = Some(lit.value());
                    Ok(())
                } else {
                    Err(meta.error(
                        "unrecognized string_enum variant attribute; expected `name`",
                    ))
                }
            })?;
            return Ok(name);
        }
    }
    Ok(None)
}

fn apply_rename(ident: &str, rename_all: Option<&str>) -> String {
    match rename_all {
        Some("UPPERCASE") => ident.to_uppercase(),
        _ => ident.to_lowercase(),
    }
}
