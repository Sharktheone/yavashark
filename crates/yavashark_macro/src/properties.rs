use proc_macro::TokenStream as TokenStream1;

use crate::config::Config;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{FnArg, ImplItem, Path};

#[derive(Debug)]
struct Item {
    name: Ident,
    attributes: Option<Attributes>,
    rename: Option<Path>,
    has_realm: bool,
    has_this: bool,
    get: Option<Ident>,
    set: Option<Ident>,
    span: Span,
}

#[allow(unused)]
pub fn properties(_: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let mut item: syn::ItemImpl = syn::parse_macro_input!(item);

    let mut new = None;

    let config = Config::new(item.span());

    let variable = config.variable;
    let native_function = config.native_function;
    let native_constructor = config.native_constructor;
    let value = config.value;
    let object = config.object;
    let object_handle = config.object_handle;
    let error = config.error;

    let mut constructor = None;
    let mut properties: Vec<Item> = Vec::new();

    for func in &mut item.items {
        let ImplItem::Fn(func) = func else {
            continue;
        };

        let mut remove = Vec::new();

        'func_attrs: for (idx, attr) in func.attrs.iter().enumerate() {
            if attr.path().is_ident("constructor") {
                let mut self_mut = false;

                if let Some(FnArg::Receiver(self_arg)) = func.sig.inputs.first() {
                    if self_arg.mutability.is_some() {
                        self_mut = true;
                    }
                }

                let mut raw = false;
                let mut special = false;

                let _ = attr.parse_nested_meta(|a| {
                    if a.path.is_ident("raw") {
                        raw = true;
                    }

                    if a.path.is_ident("special") {
                        special = true;
                    }

                    Ok(())
                });

                constructor = Some((func.sig.ident.clone(), self_mut, raw, special));
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
                    if prop.name == func.sig.ident {
                        prop.attributes = Some(attrs);
                        continue 'func_attrs;
                    }
                }

                let mut self_mut = false;

                if let Some(FnArg::Receiver(self_arg)) = func.sig.inputs.first() {
                    if self_arg.mutability.is_some() {
                        self_mut = true;
                    }
                }

                remove.push(idx);
                properties.push(Item {
                    name: func.sig.ident.clone(),
                    attributes: Some(attrs),
                    rename: None,
                    has_realm: false,
                    has_this: false,
                    get: None,
                    set: None,
                    span: attr.span(),
                });
            }
            if attr.path().is_ident("prop")
                || attr.path().is_ident("get")
                || attr.path().is_ident("set")
            {
                for prop in &mut properties {
                    if prop.name == func.sig.ident {
                        return syn::Error::new(attr.span(), "Duplicate prop attribute")
                            .to_compile_error()
                            .into();
                    }
                }

                let rename = if attr.path().is_ident("prop") {
                    attr.parse_args::<Path>().ok()
                } else {
                    None
                };

                let mut self_mut = false;

                if let Some(FnArg::Receiver(self_arg)) = func.sig.inputs.first() {
                    if self_arg.mutability.is_some() {
                        self_mut = true;
                    }
                }

                let mut has_realm = false;
                let mut has_this = false;

                let mut assert_last_or_this = false;
                let mut assert_last = false;

                func.sig.inputs.iter().for_each(|arg| {
                    if let FnArg::Typed(arg) = arg {
                        match &*arg.ty {
                            syn::Type::Reference(r) => {
                                if let syn::Type::Path(p) = &*r.elem {
                                    if p.path.is_ident("Realm") {
                                        if assert_last {
                                            panic!("this must be the last argument");
                                        }
                                        has_realm = true;
                                        assert_last_or_this = true;
                                        return;
                                    }

                                    if assert_last || assert_last_or_this {
                                        panic!("this or context must be the last argument");
                                    }
                                }
                            }

                            syn::Type::Path(p) => {
                                if p.path.is_ident("Value") {
                                    has_this = true;
                                    assert_last = true;
                                    return;
                                }

                                if assert_last {
                                    panic!("this or context must be the last argument");
                                }
                            }

                            _ => {}
                        }
                    }
                });
                let mut get = None;

                if attr.path().is_ident("get") {
                    if let Err(e) = attr.parse_nested_meta(|attr| {
                        get = Some(attr.path.require_ident()?.clone());

                        Ok(())
                    }) {
                        return e.to_compile_error().into();
                    }
                }

                let mut set = None;

                if attr.path().is_ident("set") {
                    if let Err(e) = attr.parse_nested_meta(|attr| {
                        get = Some(attr.path.require_ident()?.clone());

                        Ok(())
                    }) {
                        return e.to_compile_error().into();
                    }

                    remove.push(idx);
                }

                remove.push(idx);

                properties.push(Item {
                    name: func.sig.ident.clone(),
                    attributes: None,
                    rename,
                    has_realm,
                    has_this,
                    span: attr.span(),
                    get,
                    set,
                });
            }
            if attr.path().is_ident("new") {
                let mut n = func.sig.ident.clone();
                new = Some(n);

                remove.push(idx);
                continue;
            }
        }

        for idx in remove.into_iter().rev() {
            func.attrs.remove(idx);
        }
    }

    let mut props = TokenStream::new();

    for prop in properties {
        let name = &prop.name;
        let attrs = prop.attributes.as_ref().unwrap_or(&Attributes {
            writable: true,
            enumerable: false,
            configurable: false,
        });

        let writable = attrs.writable;
        let enumerable = attrs.enumerable;
        let configurable = attrs.configurable;

        let realm = if prop.has_realm {
            quote! {, realm }
        } else {
            TokenStream::new()
        };

        let this = if prop.has_this {
            quote! {, this }
        } else {
            TokenStream::new()
        };

        let copy = if prop.has_this {
            quote! { .copy() }
        } else {
            TokenStream::new()
        };

        let fn_name = prop
            .rename
            .as_ref()
            .map(|name| {
                quote! {
                    #name
                }
            })
            .unwrap_or(quote! {
                stringify!(#name)
            });

        let any_cast = quote! {{
            let x = x.get();
            let deez = (**x).as_any().downcast_ref::<Self>()
                .ok_or(Error::ty_error(format!("Function {:?} was not called with a valid this value: {:?} trace: {}", #fn_name, this, x.class_name())))?;

            deez.#name(args #realm #this)
        }};

        if prop.get.is_some() && prop.set.is_some() {
            return syn::Error::new(prop.span, "cannot have set and get in on the same function")
                .to_compile_error()
                .into();
        }

        let def = if let Some(name) = prop.get {
            quote! {
                obj.define_getter(stringify!(#name).into(), function)?;
            }
        } else if let Some(name) = prop.set {
            quote! {
                obj.define_setter(stringify!(#name).into(), function)?;
            }
        } else {
            quote! {
                obj.define_variable(
                    #fn_name.into(),
                    #variable::new_with_attributes(
                        function,
                        #writable,
                        #enumerable,
                        #configurable
                    )
                );
            }
        };

        let prop = quote! {
            let function = #native_function::with_proto(stringify!(#name), |args, this, realm| {
                match this #copy {
                    #value::Object(ref x) => #any_cast,
                    _ => Err(Error::ty_error(format!("Function {:?} was not called with a valid this value: {:?}", #fn_name, this))),
                }
            }, func_proto.copy()).into();

            #def
        };

        props.extend(prop);
    }

    let mut construct = TokenStream::new();

    if let Some((constructor, mutability, raw, special)) = constructor {
        let create = if special {
            quote! {
                special_with_proto
            }
        } else {
            quote! {
                with_proto
            }
        };

        let constructor_fn = if raw {
            quote! {
                let constructor_function: #value = #native_function::#create("constructor", |args, this, realm| {
                    Self::#constructor(args, this, realm)
                }, func_proto.copy()).into();
            }
        } else {
            quote! {
                let constructor_function: #value = #native_function::#create("constructor", |args, mut this, realm| {
                    if let #value::Object(x) = this {
                        let mut x = x.get();
                        let mut deez = (**x).as_any().downcast_ref::<Self>()
                            .ok_or(Error::ty_error(format!("Function {:?} was not called with a valid this value", "constructor")))?;
                        deez.#constructor(args)?;
                    }

                    Ok(Value::Undefined)

                }, func_proto.copy()).into();
            }
        };

        let new = if let Some(new) = new {
            quote! {
                Some(Box::new(Self::#new))
            }
        } else {
            quote! {
                None
            }
        };

        let prop = quote! {
            #constructor_fn

            let function: #value = #native_constructor::#create("constructor".to_string(), move || {
                    constructor_function.copy()
            }, #new, obj.clone().into(), func_proto.copy()).into();


            function.define_property("prototype".into(), obj.clone().into())?;

            obj.define_variable(
                "constructor".into(),
                #variable::new_with_attributes(
                    function,
                    true,
                    false,
                    false
                )
            )?;
        };

        construct.extend(prop);
    }

    let new_fn = quote! {
        pub(crate) fn initialize_proto(mut obj: #object, func_proto: #value) -> Result<#object_handle, #error> {
            use yavashark_value::{AsAny, Obj};
            #props

            let obj = obj.into_object();

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
