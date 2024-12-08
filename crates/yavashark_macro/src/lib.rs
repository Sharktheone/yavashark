extern crate proc_macro;

use std::env;

mod config;
mod obj;
mod properties;
mod props;

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

fn env_path() -> syn::Path {
    let name = env::var("CARGO_PKG_NAME").unwrap_or("".to_string());
    if name == "yavashark_env" {
        syn::parse_str("crate").unwrap()
    } else {
        syn::parse_str("yavashark_env").unwrap()
    }
}
