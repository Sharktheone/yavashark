use proc_macro::TokenStream as TokenStream1;

use proc_macro2::{Ident, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{FnArg, ImplItem, LitBool, Path, PathSegment};

#[derive(Debug)]
struct Item {
    name: Ident,
    attributes: Option<Attributes>,
    rename: Option<Path>,
    is_mut: bool,
    has_ctx: bool,
    has_this: bool,
}

#[allow(unused)]
pub fn properties(_: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let mut item: syn::ItemImpl = syn::parse_macro_input!(item);

    let mut call = None;
    let mut constructor = None;
    let mut new = None;

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

    let mut object_handle = crate_path.clone();
    object_handle
        .segments
        .push(PathSegment::from(Ident::new("ObjectHandle", item.span())));

    let mut object = crate_path.clone();
    object
        .segments
        .push(PathSegment::from(Ident::new("object", item.span())));
    object
        .segments
        .push(PathSegment::from(Ident::new("Object", item.span())));

    let mut value = crate_path;
    value
        .segments
        .push(PathSegment::from(Ident::new("Value", item.span())));

    struct Property {
        name: Ident,
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

                let _ = attr.parse_nested_meta(|a| {
                    if a.path.is_ident("raw") {
                        raw = true;
                    }

                    Ok(())
                });

                constructor = Some((func.sig.ident.clone(), self_mut, raw));
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
                    is_mut: false,
                    has_ctx: false,
                    has_this: false,
                });
            }
            if attr.path().is_ident("prop") {
                for prop in &mut properties {
                    if prop.name == func.sig.ident {
                        return syn::Error::new(attr.span(), "Duplicate prop attribute")
                            .to_compile_error()
                            .into();
                    }
                }

                let rename = attr.parse_args::<Path>().ok();

                let mut self_mut = false;

                if let Some(FnArg::Receiver(self_arg)) = func.sig.inputs.first() {
                    if self_arg.mutability.is_some() {
                        self_mut = true;
                    }
                }

                let mut has_ctx = false;
                let mut has_this = false;

                let mut assert_last_or_this = false;
                let mut assert_last = false;

                func.sig.inputs.iter().for_each(|arg| {
                    if let FnArg::Typed(arg) = arg {
                        match &*arg.ty {
                            syn::Type::Reference(r) => {
                                if let syn::Type::Path(p) = &*r.elem {
                                    if p.path.is_ident("Context") {
                                        if assert_last {
                                            panic!("this must be the last argument");
                                        }
                                        has_ctx = true;
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

                remove.push(idx);

                properties.push(Item {
                    name: func.sig.ident.clone(),
                    attributes: None,
                    rename,
                    is_mut: self_mut,
                    has_ctx,
                    has_this,
                });
            }
            if attr.path().is_ident("new") {
                let mut n = (func.sig.ident.clone(), false);
                
                
                attr.parse_nested_meta(|a| {
                    if a.path.is_ident("this") {
                        n.1 = true;
                        return Ok(());
                    }
                    
                    return Err(syn::Error::new(a.input.span(), "Unknown attribute"));
                });
                
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

        let ctx = if prop.has_ctx {
            quote! {, ctx }
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
            .map(ToTokens::to_token_stream)
            .unwrap_or(quote! {
                stringify!(#name)
            });

        let any_cast = if prop.is_mut {
            quote! {{
                let mut x = x.get_mut()?;
                let mut deez = (***x).as_any_mut().downcast_mut::<Self>()
                    .ok_or(Error::ty_error(format!("Function {:?} was not called with a valid this value", #fn_name)))?;
                deez.#name(args, ctx)
            }}
        } else {
            quote! {{
                let x = x.get()?;
                let deez = (***x).as_any().downcast_ref::<Self>()
                    .ok_or(Error::ty_error(format!("Function {:?} was not called with a valid this value: {:?}", #fn_name, this)))?;

                deez.#name(args #ctx #this)
            }}
        };

        let prop = quote! {
            let function = #native_function::with_proto(stringify!(#name), |args, this, ctx| {
                match this #copy {
                    #value::Object(x) => #any_cast,
                    _ => Err(Error::ty_error(format!("Function {:?} was not called with a valid this value: {:?}", #fn_name, this))),
                }
            }, func_proto.copy()).into();

            obj.define_variable(
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
    }

    let mut construct = TokenStream::new();

    if let Some((constructor, mutability, raw)) = constructor {

        let constructor_fn = if raw {
            quote! {
                let function: #value = #native_function::with_proto("constructor", |args, this, ctx| {
                    Self::#constructor(args, this, ctx)
                }, func_proto.copy()).into();
            }

        } else {
            let (new, req_this) = new.expect("Object with constructor must have a method annotated with #[new]");

            
            let req_this = if req_this {
                quote! { Some(this.copy()) }
            } else {
                quote! { None }
            };
            
            quote! {
                let function: #value = #native_function::with_proto("constructor", |args, mut this, ctx| {
                    let mut new = Self::#new(ctx, #req_this)?;
                    new.#constructor(args)?;

                    let boxed: Box<dyn Obj<Context>> = Box::new(new);

                    this.exchange(boxed);

                    Ok(Value::Undefined)

                }, func_proto.copy()).into();
            }
        };



        let prop = quote! {
            #constructor_fn
            function.define_property("prototype".into(), obj.clone().into())?;

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
