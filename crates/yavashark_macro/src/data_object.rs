mod enumeration;
mod structure;

use crate::data_object::enumeration::data_enum;
use crate::data_object::structure::data_struct;
use syn::parse::Parse;
use syn::{Data, DeriveInput, Token};

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
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![struct]) {
            let item_struct: syn::ItemStruct = input.parse()?;
            Ok(StructOrEnum::Struct(item_struct))
        } else if lookahead.peek(Token![enum]) {
            let item_enum: syn::ItemEnum = input.parse()?;
            Ok(StructOrEnum::Enum(item_enum))
        } else {
            Err(lookahead.error())
        }
    }
}
