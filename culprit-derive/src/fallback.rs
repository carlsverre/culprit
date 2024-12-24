use proc_macro2::TokenStream;
use quote::quote;
use syn::DeriveInput;

/// This function provides fallback implementations for `Error` and `Display`
/// for a type along with a `syn::Error`. The purpose is to minimize spurious
/// distracting compile errors.
pub fn expand(input: &DeriveInput, error: syn::Error) -> TokenStream {
    let ty = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let error = error.to_compile_error();

    quote! {
        #error

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::error::Error for #ty #ty_generics #where_clause
        where
            // Work around trivial bounds being unstable.
            // https://github.com/rust-lang/rust/issues/48214
            for<'workaround> #ty #ty_generics: ::core::fmt::Debug,
        {}

        #[allow(unused_qualifications)]
        #[automatically_derived]
        impl #impl_generics ::core::fmt::Display for #ty #ty_generics #where_clause {
            fn fmt(&self, __formatter: &mut ::core::fmt::Formatter) -> ::core::fmt::Result {
                ::core::unreachable!()
            }
        }
    }
}
