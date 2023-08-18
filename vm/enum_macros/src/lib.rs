extern crate proc_macro;

use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput};

#[proc_macro_derive(TryFromByte)]
pub fn try_from_primitive_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let name = &input.ident;

    let variants = generate_variants(&input.data);

    let match_arms: Vec<_> = variants
        .into_iter()
        .map(|(idx, variant_name)| {
            quote! {
                #idx => Ok(#name::#variant_name),
            }
        })
        .collect();

    let expanded = quote! {
        impl std::convert::TryFrom<u8> for #name {
            type Error = ();

            fn try_from(value: u8) -> Result<Self, Self::Error> {
                match value {
                    #(#match_arms)*
                    _ => Err(()),
                }
            }
        }
    };

    TokenStream::from(expanded)
}

fn generate_variants(data: &syn::Data) -> Vec<(u8, syn::Ident)> {
    match data {
        syn::Data::Enum(data_enum) => data_enum
            .variants
            .iter()
            .enumerate()
            .map(|(idx, variant)| {
                let name = &variant.ident;
                match &variant.fields {
                    syn::Fields::Unit => (idx as u8, name.clone()),
                    _ => panic!("Only unit variants are supported!"),
                }
            })
            .collect(),
        _ => panic!("Only enums are supported!"),
    }
}
