extern crate proc_macro;

mod obj;
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
    props::properties(attrs, item)
}
