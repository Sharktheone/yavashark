use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;

#[allow(unused)]
pub fn properties(_: TokenStream1, item: TokenStream1) -> TokenStream1 {
    TokenStream::new().into()
}
