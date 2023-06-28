use proc_macro::TokenStream;
use std::fmt::Display;
use proc_macro2::Span;
use quote::quote;
use syn::{Data, DeriveInput, Error, parse_macro_input};

fn derive_error<T: Display>(message: T) -> TokenStream {
    Error::new(Span::call_site(), message)
            .to_compile_error()
            .into()
}

#[proc_macro_derive(EnumDispatchClone)]
pub fn derive(input: TokenStream) -> TokenStream {
    let DeriveInput { ident, data, .. } = parse_macro_input!(input);
    let Data::Enum(enum_data) = data else {
        return derive_error("EnumDispatchClone only works on enums");
    };

    let match_cases = enum_data.variants.iter().map(|variant| {
        let variant_ident = &variant.ident;
        quote! {
            #ident::#variant_ident(v) => { v.clone().into() }
        }
    });

    let match_cases = quote! {
        #( #match_cases ),*
    };

    let output = quote! {
        impl ::core::clone::Clone for #ident {
            fn clone(&self) -> Self {
                match self {
                    #match_cases
                }
            }
        }
    };
    output.into()
}