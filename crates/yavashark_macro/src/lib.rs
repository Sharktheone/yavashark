mod obj;

extern crate proc_macro;


#[proc_macro_attribute]
pub fn object(attrs: proc_macro::TokenStream, item: proc_macro::TokenStream) -> proc_macro::TokenStream {
    obj::object(attrs, item)
}