use crate::config::Config;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::Path;

pub fn custom_props(attrs: TokenStream1, item: TokenStream1) -> TokenStream1 {
    item //TODO
    

#[derive(Debug, Eq, PartialEq)]
pub enum Act {
    Ref,
    // RefMut,
    None,
    Set,
    SetVar,
    Contains,
    Delete,
}

pub fn match_prop(
    properties: &Vec<(Path, Option<Path>)>,
    r: Act,
    value_path: &Path,
) -> TokenStream {
    let mut match_properties_define = TokenStream::new();
    let match_non_string = TokenStream::new();

    for (field, rename) in properties {
        let act = match r {
            Act::Ref => quote! {Some(& self.#field.value)},
            // Act::RefMut => quote! {Some(&mut self.#field.value)},
            Act::None => quote! {Some(self.#field.clone())},
            Act::Set => quote! {self.#field = value.into()},
            Act::SetVar => quote! {self.#field = value.into()},
            Act::Contains => quote! {true},
            Act::Delete => quote! {
                {
                    let old = self.#field.value.copy();
                    self.#field.value = #value_path::Undefined;
                    Some(old)
                }
            },
        };
        if let Some(rename) = rename {
            let expanded = if matches!(r, Act::Set | Act::SetVar) {
                quote! {
                    stringify!(#rename) => {
                        #act;
                        return;
                    }
                }
            } else {
                quote! {
                    stringify!(#rename) => {
                        return #act;
                    }
                }
            };

            match_properties_define.extend(expanded); //TODO: we currently don't have a way to set up a non string field
            continue;
        }

        let expanded = quote! {
            stringify!(#field) =>  {
                return #act;
            }
        };

        match_properties_define.extend(expanded);
    }

    if !match_properties_define.is_empty() {
        match_properties_define = quote! {
            if let Value::String(name) = &name {
                match name.as_str() {
                    #match_properties_define
                    _ => {}
                }
            }
        };
    }

    if !match_non_string.is_empty() {
        match_properties_define = quote! {
            #match_properties_define

            match name {
                #match_non_string
                _ => {}
            }
        };
    }

    match_properties_define
}

pub enum List {
    Properties,
    Keys,
    Values,
    Clear,
}

pub fn match_list(properties: &Vec<(Path, Option<Path>)>, r: List, value: &Path) -> TokenStream {
    let mut add = TokenStream::new();

    for (field, rename) in properties {
        let name = if let Some(rename) = rename {
            quote! { #value::string(stringify!(#rename)) }
        } else {
            quote! {
                #value::string(stringify!(#field))
            }
        };

        let act = match r {
            List::Properties => {
                quote! {props.push((#name, self.#field.value.copy()));}
            }
            List::Keys => quote! {keys.push(#name);},
            List::Values => quote! {values.push(self.#field.value.copy());},
            List::Clear => quote! {self.#field.value = #value::Undefined;},
        };

        add.extend(act);
    }

    add
}
