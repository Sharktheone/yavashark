use proc_macro::TokenStream as TokenStream1;

use proc_macro2::{Ident, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::{FieldMutability, Fields, Path, PathSegment};

pub fn object(attrs: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let mut input: syn::ItemStruct = syn::parse_macro_input!(item);
    let mut proto = false;
    let mut direct = Vec::new();
    let mut constructor = false;

    let span = input.span();

    let crate_path = Path::from(Ident::new("crate", input.span()));

    let mut obj_path = crate_path.clone();
    obj_path
        .segments
        .push(PathSegment::from(Ident::new("object", input.span())));
    obj_path
        .segments
        .push(PathSegment::from(Ident::new("Object", input.span())));

    let mut variable = crate_path.clone();
    variable
        .segments
        .push(PathSegment::from(Ident::new("Variable", input.span())));

    let mut context = crate_path.clone();
    context
        .segments
        .push(PathSegment::from(Ident::new("Context", input.span())));

    let mut value = crate_path.clone();
    value
        .segments
        .push(PathSegment::from(Ident::new("Value", input.span())));

    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("prototype") {
            proto = true;
            return Ok(());
        }
        if meta.path.is_ident("direct") {
            meta.parse_nested_meta(|meta| {
                let mut rename = None;

                let _ = meta.parse_nested_meta(|meta| {
                    rename = Some(meta.path);
                    Ok(())
                });

                direct.push((meta.path, rename));

                Ok(())
            })?;
            return Ok(());
        }
        if meta.path.is_ident("object") {
            obj_path = meta.path;
            return Ok(());
        }

        if meta.path.is_ident("variable") {
            variable = meta.path;
            return Ok(());
        }

        if meta.path.is_ident("context") {
            context = meta.path;
            return Ok(());
        }

        if meta.path.is_ident("value") {
            value = meta.path;
            return Ok(());
        }

        if meta.path.is_ident("constructor") {
            constructor = true;
            return Ok(());
        }

        Err(syn::Error::new(meta.path.span(), "Unknown attribute"))
    });

    syn::parse_macro_input!(attrs with attr_parser);

    let Fields::Named(fields) = &mut input.fields else {
        return syn::Error::new(input.span(), "Object must have named fields")
            .to_compile_error()
            .into();
    };

    fields.named.push(syn::Field {
        attrs: Vec::new(),
        vis: syn::Visibility::Inherited,
        mutability: FieldMutability::None,
        ident: Some(Ident::new("object", span)),
        colon_token: None,
        ty: syn::Type::Path(syn::TypePath {
            qself: None,
            path: obj_path.clone(),
        }),
    });

    if constructor {
        direct.push((Ident::new("constructor", span).into(), None));
    }

    for (path, _) in &direct {
        fields.named.push(syn::Field {
            attrs: Vec::new(),
            vis: syn::Visibility::Inherited,
            mutability: FieldMutability::None,
            ident: path.get_ident().cloned(),
            colon_token: None,
            ty: syn::Type::Path(syn::TypePath {
                qself: None,
                path: variable.clone(),
            }),
        });
    }

    let struct_name = &input.ident;

    let properties_define = match_prop(&direct, Act::Set);
    let properties_variable_define = match_prop(&direct, Act::SetVar);
    let properties_resolve = match_prop(&direct, Act::None);
    let properties_get = match_prop(&direct, Act::Ref);
    let properties_get_mut = match_prop(&direct, Act::RefMut);
    let properties_contains = match_prop(&direct, Act::Contains);

    let properties = match_list(&direct, List::Properties, &value);
    let keys = match_list(&direct, List::Keys, &value);
    let values = match_list(&direct, List::Values, &value);

    let expanded = quote! {
        #input

        impl yavashark_value::Obj<#context> for #struct_name {
            fn define_property(&mut self, name: #value, value: #value) {
                #properties_define
                self.object.define_property(name, value);
            }

            fn define_variable(&mut self, name: #value, value: #variable) {
                #properties_variable_define

                self.object.define_variable(name, value);
            }

            fn resolve_property(&self, name: &#value) -> Option<#value> {
                #properties_resolve
                self.object.resolve_property(name)
            }

            fn get_property(&self, name: &#value) -> Option<&#value> {
                #properties_get
                self.object.get_property(name)
            }

            fn get_property_mut(&mut self, name: &#value) -> Option<&mut #value> {
                #properties_get_mut
                self.object.get_property_mut(name)
            }

            fn contains_key(&self, name: &#value) -> bool {
                #properties_contains
                self.object.contains_key(name)
            }

            fn name(&self) -> String {
                self.object.name()
            }

            fn to_string(&self) -> String {
                self.object.to_string()
            }

            fn properties(&self) -> Vec<(#value, #value)> {
                let mut props = self.object.properties();
                #properties
                props
            }

            fn keys(&self) -> Vec<#value> {
                let mut keys = self.object.keys();
                #keys
                keys
            }

            fn values(&self) -> Vec<#value> {
                let mut values = self.object.values();
                #values
                values
            }
        }
    };

    TokenStream1::from(expanded)
}

enum Act {
    Ref,
    RefMut,
    None,
    Set,
    SetVar,
    Contains,
}

fn match_prop(properties: &Vec<(Path, Option<Path>)>, r: Act) -> TokenStream {
    let mut match_properties_define = TokenStream::new();
    let mut match_non_string = TokenStream::new();

    for (field, rename) in properties {
        let act = match r {
            Act::Ref => quote! {Some(& self.#field.value)},
            Act::RefMut => quote! {Some(&mut self.#field.value)},
            Act::None => quote! {Some(self.#field.value.copy())},
            Act::Set => quote! {self.#field = value.into()},
            Act::SetVar => quote! {self.#field = value},
            Act::Contains => quote! {true},
        };
        if let Some(rename) = rename {
            let expanded = if matches!(r, Act::Set | Act::SetVar) {
                quote! {
                    #rename => {
                        self.#field = value.into();
                        return;
                    }
                }
            } else {
                quote! {
                    & #rename => {
                        return #act;
                    }
                }
            };

            match_non_string.extend(expanded);
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

enum List {
    Properties,
    Keys,
    Values,
}

fn match_list(properties: &Vec<(Path, Option<Path>)>, r: List, value: &Path) -> TokenStream {
    let mut add = TokenStream::new();

    for (field, rename) in properties {
        let name = if let Some(rename) = rename {
            quote! { #rename }
        } else {
            quote! {
                #value::string(stringify!(#field))
            }
        };

        let act = match r {
            List::Properties => {
                quote! {props.push((#name, self.#field.copy()));}
            }
            List::Keys => quote! {keys.push(#name);},
            List::Values => quote! {values.push(self.#field.copy());},
        };

        add.extend(act);
    }

    add
}
