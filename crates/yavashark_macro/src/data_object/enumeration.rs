use proc_macro2::TokenStream;

pub fn data_enum(e: syn::ItemEnum) -> syn::Result<TokenStream> {
    Ok(TokenStream::new())
}
