mod enumeration;
mod structure;

use syn::parse::discouraged::Speculative;
use crate::data_object::enumeration::data_enum;
use crate::data_object::structure::data_struct;
use syn::parse::Parse;
use syn::spanned::Spanned;
use syn::Token;

pub fn data_object(
    _attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as StructOrEnum);

    match input {
        StructOrEnum::Struct(s) => data_struct(s),
        StructOrEnum::Enum(e) => data_enum(e),
    }
    .unwrap_or_else(|e| e.to_compile_error())
    .into()
}

pub enum StructOrEnum {
    Struct(syn::ItemStruct),
    Enum(syn::ItemEnum),
}

impl Parse for StructOrEnum {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let fork = input.fork();
        let item: syn::Item = fork.parse()?;
        match item {
            syn::Item::Struct(s) => {
                input.advance_to(&fork);
                Ok(StructOrEnum::Struct(s))
            }
            syn::Item::Enum(e) => {
                input.advance_to(&fork);
                Ok(StructOrEnum::Enum(e))
            }
            _ => Err(syn::Error::new(item.span(), "expected struct or enum")),
        }
    }
}
