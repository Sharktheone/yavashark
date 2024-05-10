use proc_macro::TokenStream as TokenStream1;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{ImplItem, LitBool, Path, PathSegment};

#[allow(unused)]
pub fn properties(_: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let mut item: syn::ItemImpl = syn::parse_macro_input!(item);

    let mut call = None;
    let mut constructor = None;

    let crate_path = Path::from(Ident::new("crate", item.span()));

    let mut context = crate_path.clone();
    context
        .segments
        .push(PathSegment::from(Ident::new("Context", item.span())));

    let mut error = crate_path.clone();
    error
        .segments
        .push(PathSegment::from(Ident::new("Error", item.span())));

    let mut native_function = crate_path.clone();
    native_function
        .segments
        .push(PathSegment::from(Ident::new("NativeFunction", item.span())));

    let mut variable = crate_path.clone();
    variable
        .segments
        .push(PathSegment::from(Ident::new("Variable", item.span())));

    let mut object = crate_path.clone();
    object
        .segments
        .push(PathSegment::from(Ident::new("ObjectHandle", item.span())));

    let mut value = crate_path.clone();
    value
        .segments
        .push(PathSegment::from(Ident::new("Value", item.span())));

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

    let mut constructor = None;
    let mut properties: Vec<(Ident, Option<Attributes>, Option<Path>)> = Vec::new();

    for func in &mut item.items {
        let ImplItem::Fn(func) = func else {
            continue;
        };

        let mut remove = Vec::new();

        'func_attrs: for (idx, attr) in func.attrs.iter().enumerate() {
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
                properties.push((func.sig.ident.clone(), Some(attrs), None));
            }
            if attr.path().is_ident("prop") {
                for prop in &mut properties {
                    if prop.0 == func.sig.ident {
                        return syn::Error::new(attr.span(), "Duplicate prop attribute")
                            .to_compile_error()
                            .into();
                    }
                }

                let rename = attr.parse_args::<Path>().ok();

                remove.push(idx);
                properties.push((func.sig.ident.clone(), None, rename));
            }
        }

        for idx in remove.into_iter().rev() {
            func.attrs.remove(idx);
        }
    }

    let mut props = TokenStream::new();

    for prop in properties {
        let name = &prop.0;
        let attrs = prop.1.as_ref().unwrap_or(&Attributes {
            writable: true,
            enumerable: false,
            configurable: false,
        });

        let writable = attrs.writable;
        let enumerable = attrs.enumerable;
        let configurable = attrs.configurable;

        let fn_name = prop
            .2
            .as_ref()
            .map(|i| i.to_token_stream())
            .unwrap_or(quote! {
                stringify!(#name)
            });

        let prop = quote! {
            let function = #native_function::new(stringify!(#name), |args, this| {
                let deez = this.as_any().downcast_ref::<Self>()
                    .ok_or(Error::ty_error(format!("Function {:?} was not called with the a this value", #fn_name)))?;

                deez.#name(args)
            }, ctx).into();

            self.define_variable(
                #fn_name.into(),
                #variable::new_with_attributes(
                    function,
                    #writable,
                    #enumerable,
                    #configurable
                )
            );
        };

        props.extend(prop);

        //TODO
    }

    let mut construct = TokenStream::new();

    if let Some(constructor) = constructor {
        let prop = quote! {
            let function: #value = #native_function::new("constructor", |args, this| {
                let deez = this.as_any().downcast_ref::<Self>()
                    .ok_or(Error::ty("Function constructor was not called with the a this value"))?;

                deez.#constructor(args)
            }, ctx).into();

            function.define_property("prototype".into(), obj.clone().into());

            obj.define_variable(
                "constructor".into(),
                #variable::new_with_attributes(
                    function,
                    true,
                    false,
                    false
                )
            );
        };

        construct.extend(prop);
    }

    let new_fn = quote! {
        fn initialize(mut self, ctx: &mut #context) -> Result<#object, #error> {
            use yavashark_value::{AsAny, Obj};
            #props

            let obj: #object = self.into_object();

            #construct

            Ok(obj)
        }
    };

    let new_fn = ImplItem::Verbatim(new_fn);

    item.items.push(new_fn);

    item.to_token_stream().into()
}

#[derive(Debug)]
struct Attributes {
    writable: bool,
    enumerable: bool,
    configurable: bool,
}
