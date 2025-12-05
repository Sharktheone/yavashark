mod args;
mod enumeration;
mod structure;

use crate::data_object::args::DataObjectArgs;
use crate::data_object::enumeration::data_enum;
use crate::data_object::structure::data_struct;
use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream as TokenStream1;
use syn::parse::discouraged::Speculative;
use syn::parse::Parse;
use syn::spanned::Spanned;

pub fn data_object(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let input = syn::parse_macro_input!(item as StructOrEnum);

    let attr_args = match NestedMeta::parse_meta_list(attrs.into()) {
        Ok(v) => v,
        Err(e) => {
            return TokenStream1::from(darling::Error::from(e).write_errors());
        }
    };

    let args = match DataObjectArgs::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => return e.write_errors().into(),
    };

    match input {
        StructOrEnum::Struct(s) => data_struct(s),
        StructOrEnum::Enum(e) => data_enum(e, args),
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
