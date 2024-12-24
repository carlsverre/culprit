use proc_macro2::TokenStream;
use quote::quote;
use syn::{DeriveInput, Result};

pub fn culprit_try_derive(input: &DeriveInput) -> Result<TokenStream> {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    Ok(quote! {
        // ...
    })
}
