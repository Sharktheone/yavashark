use crate::config::Config;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::Path;

pub fn custom_props(attrs: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let conf = Config::new(Span::call_site());

    let mut direct = Vec::new();

    let parser = syn::meta::parser(|meta| {
        let mut rename = None;

        let _ = meta.parse_nested_meta(|meta| {
            rename = Some(meta.path);
            Ok(())
        });

        direct.push((meta.path, rename));

        Ok(())
    });

    syn::parse_macro_input!(attrs with parser);

    if direct.is_empty() {
        return item;
    }

    let mut item: syn::ItemImpl = syn::parse_macro_input!(item);

    let value = &conf.value;
    let variable = &conf.variable;
    let obj_prop = &conf.object_property;

    let properties_define = match_prop(&direct, Act::Set, value);

    item.items.push(syn::parse_quote! {
        fn define_property(&mut self, name: Value, value: Value) {
            #properties_define

            self.get_wrapped_object_mut().define_property(name, value);
        }
    });

    let properties_variable_define = match_prop(&direct, Act::SetVar, value);

    item.items.push(syn::parse_quote! {
        fn define_variable(&mut self, name: Value, value: #variable) {
            #properties_variable_define

            self.get_wrapped_object_mut().define_variable(name, value);
        }
    });

    let properties_resolve = match_prop(&direct, Act::None, value);

    item.items.push(syn::parse_quote! {
        fn resolve_property(&self, name: & #value) -> Option<#obj_prop> {
            #properties_resolve

            self.get_wrapped_object().resolve_property(name)
        }
    });

    let properties_get = match_prop(&direct, Act::Ref, value);

    item.items.push(syn::parse_quote! {
        fn get_property(&self, name: & #value) -> Option<& #value> {
            #properties_get

            self.get_wrapped_object().get_property(name)
        }
    });

    let properties_delete = match_prop(&direct, Act::Delete, value);

    item.items.push(syn::parse_quote! {
        fn delete_property(&mut self, name: &Value) -> Option<Value> {
            #properties_delete

            self.get_wrapped_object_mut().delete_property(name)
        }
    });

    let properties_contains = match_prop(&direct, Act::Contains, value);

    item.items.push(syn::parse_quote! {
        fn contains_key(&self, name: & #value) -> bool {
            #properties_contains

            self.get_wrapped_object().contains_key(name)
        }
    });

    let properties = match_list(&direct, List::Properties, value);

    item.items.push(syn::parse_quote! {
        fn properties(&self) -> Vec<(#value, #value)> {
            let mut props = self.get_wrapped_object().properties();

            #properties

            props
        }
    });

    let keys = match_list(&direct, List::Keys, value);

    item.items.push(syn::parse_quote! {
        fn keys(&self) -> Vec<#value> {
            let mut keys = self.get_wrapped_object().keys();

            #keys

            keys
        }
    });

    let values = match_list(&direct, List::Values, value);

    item.items.push(syn::parse_quote! {
        fn values(&self) -> Vec<#value> {
            let mut values = self.get_wrapped_object().values();

            #values

            values
        }
    });

    let clear = match_list(&direct, List::Clear, value);

    item.items.push(syn::parse_quote! {
        fn clear_values(&mut self) {
            self.get_wrapped_object_mut().clear_values();

            #clear
        }
    });

    item.to_token_stream().into()
}

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
            if let #value_path::String(name) = &name {
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
