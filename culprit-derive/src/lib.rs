use proc_macro::TokenStream;
use syn::{parse_macro_input, DeriveInput};

mod expand;
mod fallback;

#[proc_macro_derive(Culprit)]
pub fn culprit_derive(input: TokenStream) -> TokenStream {
    // Parse the input tokens into a syntax tree
    let input = parse_macro_input!(input as DeriveInput);
    match expand::culprit_try_derive(&input) {
        Ok(stream) => stream.into(),
        Err(err) => fallback::expand(&input, err).into(),
    }
}
