extern crate proc_macro;

use std::env;

mod config;
mod custom_props;
mod mutable_region;
mod obj;
mod properties;
mod properties_new;
mod props;
mod typed_array;
mod instruction;

#[proc_macro_attribute]
pub fn object(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    obj::object(attrs, item)
}

#[proc_macro_attribute]
pub fn properties(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    properties::properties(attrs, item)
}

#[proc_macro_attribute]
pub fn properties_new(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    props::properties(attrs, item)
}

#[proc_macro_attribute]
pub fn props(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    properties_new::properties(attrs, item).unwrap_or_else(|e| e.to_compile_error().into())
}

#[proc_macro_attribute]
pub fn custom_props(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    custom_props::custom_props(attrs, item)
}


#[proc_macro]
pub fn typed_array_run(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let output = typed_array::typed_array_run(input.into());
    output.into()
}

#[proc_macro]
pub fn typed_array_run_mut(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let output = typed_array::typed_array_run_mut(input.into());
    output.into()
}


#[proc_macro]
pub fn inst(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    instruction::instruction(input.into()).map(Into::into).unwrap_or_else(|e| e.to_compile_error().into())
}

fn env_path() -> syn::Path {
    let name = env::var("CARGO_PKG_NAME").unwrap_or("".to_string());
    if name == "yavashark_env" {
        syn::parse_str("crate").unwrap()
    } else {
        syn::parse_str("yavashark_env").unwrap()
    }
}

fn value_path() -> syn::Path {
    let name = env::var("CARGO_PKG_NAME").unwrap_or("".to_string());
    if name == "yavashark_value" {
        syn::parse_str("crate").unwrap()
    } else {
        syn::parse_str("yavashark_value").unwrap()
    }
}
