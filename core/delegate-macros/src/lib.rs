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
