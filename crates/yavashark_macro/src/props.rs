use proc_macro::TokenStream as TokenStream1;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::{ImplItem, ImplItemFn, LitBool, Visibility};
use syn::spanned::Spanned;

pub fn properties(_: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let mut item: syn::ItemImpl = syn::parse_macro_input!(item);

    let mut call = None;
    let mut constructor = None;

    struct Property {
        name: syn::Ident,
        writable: bool,
        enumerable: bool,
        configurable: bool,
    }

    let mut properties = Vec::new();

    for func in &item.items {
        let ImplItem::Fn(func) = func else {
            continue;
        };

        let mut writable = false;
        let mut enumerable = false;
        let mut configurable = false;

        let mut name = None;

        for attr in &func.attrs {
            if attr.path().is_ident("call") {
                call = Some(func.sig.ident.clone());
                continue;
            }
            if attr.path().is_ident("constructor") {
                constructor = Some(func.sig.ident.clone());
                continue;
            }

            if attr.path().is_ident("attributes") {
                let attr_parser = syn::meta::parser(|meta| {
                    if meta.path.is_ident("writable") {
                        writable = meta.value()?.parse::<LitBool>()?.value();
                        return Ok(());
                    }

                    if meta.path.is_ident("enumerable") {
                        enumerable = meta.value()?.parse::<LitBool>()?.value();
                        return Ok(());
                    }

                    if meta.path.is_ident("configurable") {
                        configurable = meta.value()?.parse::<LitBool>()?.value();
                        return Ok(());
                    }

                    Err(syn::Error::new(meta.path.span(), "Unknown attribute"))
                });

                attr.parse_args_with(attr_parser)
                    .expect("Failed to parse attributes");
                continue;
            }

            if attr.path().is_ident("name") {
                name = Some(
                    attr.meta
                        .path()
                        .get_ident()
                        .expect("Expected identifier")
                        .clone(),
                );
                continue;
            }
        }

        let Some(name) = name else {
            continue;
        };

        let prop = Property {
            name,
            writable,
            enumerable,
            configurable,
        };

        properties.push(prop);
    }

    let mut call = None;
    let mut constructor = None;
    let mut properties: Vec<(Ident, Option<Attributes>)> = Vec::new();

    for func in &mut item.items {
        let ImplItem::Fn(func) = func else {
            continue;
        };

        let mut remove = Vec::new();

        'func_attrs: for (idx, attr) in func.attrs.iter().enumerate() {
            if attr.path().is_ident("call") {
                if call.is_some() {
                    return syn::Error::new(attr.span(), "Duplicate call attribute")
                        .to_compile_error()
                        .into();
                }


                call = Some(func.sig.ident.clone());
                remove.push(idx);
                continue;
            }
            if attr.path().is_ident("constructor") {
                constructor = Some(func.sig.ident.clone());
                remove.push(idx);
                continue;
            }
            if attr.path().is_ident("attributes") {
                let mut attrs = Attributes {
                    writable: true,
                    enumerable: true,
                    configurable: true,
                };

                let _ = attr.parse_nested_meta(|a| {
                    if a.path.is_ident("writable") {
                        attrs.writable = true;
                        return Ok(());
                    }

                    if a.path.is_ident("enumerable") {
                        attrs.enumerable = true;
                        return Ok(());
                    }

                    if a.path.is_ident("configurable") {
                        attrs.configurable = true;
                        return Ok(());
                    }

                    Ok(())
                });


                for prop in &mut properties {
                    if prop.0 == func.sig.ident {
                        prop.1 = Some(attrs);
                        continue 'func_attrs;
                    }
                }

                remove.push(idx);
                properties.push((func.sig.ident.clone(), Some(attrs)));
            }
            if attr.path().is_ident("prop") {
                for prop in &mut properties {
                    if prop.0 == func.sig.ident {
                        return syn::Error::new(attr.span(), "Duplicate prop attribute")
                            .to_compile_error()
                            .into();
                    }
                }

                let rename = attr.parse_args::<syn::Ident>().unwrap_or(func.sig.ident.clone());

                remove.push(idx);
                properties.push((rename, None));
            }
        }

        for idx in remove.into_iter().rev() {
            func.attrs.remove(idx);
        }
    }

    

    let props = TokenStream::new();
    
    for prop in properties {
        let name = &prop.0;
        let attrs = prop.1.as_ref().unwrap_or(&Attributes {
            writable: true,
            enumerable: true,
            configurable: true,
        });

        let writable = attrs.writable;
        let enumerable = attrs.enumerable;
        let configurable = attrs.configurable;

        let prop = quote! {
            self.
        };

        props.extend(prop);
        
        //TODO
    }
    
    
    let new_func = quote! {
        fn initialize(&mut self) {
            #props
        }
    };
    
    item.to_token_stream().into()
}


#[derive(Debug)]
struct Attributes {
    writable: bool,
    enumerable: bool,
    configurable: bool,
}