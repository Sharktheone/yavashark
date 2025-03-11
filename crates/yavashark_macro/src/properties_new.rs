mod constant;
mod method;

use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{spanned::Spanned, ImplItem, ItemImpl};

use crate::config::Config;
use crate::properties_new::constant::{parse_constant, Constant};
use crate::properties_new::method::{parse_method, Method};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Normal,
    Get,
    Set,
}

enum Prop {
    Method(Method),
    Constant(Constant),
}

enum MaybeConstructor<T> {
    Impl(T),
    Static(T),
    Constructor(T),
    CallConstructor(T),
    CallAndConstructor(T),
}

enum MaybeStatic<T> {
    Impl(T),
    Static(T),
}

#[allow(unused)]
pub fn properties(attrs: TokenStream1, item: TokenStream1) -> syn::Result<TokenStream1> {
    // Parse top-level attributes with darling:
    let mut item_impl = syn::parse::<ItemImpl>(item)?;

    let mut props = Vec::new();
    let mut static_props = Vec::new();
    let mut constructor = None;
    let mut call_constructor = None;

    for item in &mut item_impl.items {
        match item {
            ImplItem::Fn(func) => {
                let (property, _) = parse_method(func)?;
                //TODO: handle static with receiver

                match property {
                    MaybeConstructor::Impl(property) => {
                        props.push(Prop::Method(property));
                    }
                    MaybeConstructor::Static(property) => {
                        static_props.push(Prop::Method(property));
                    }
                    MaybeConstructor::Constructor(property) => {
                        constructor = Some(property);
                    }
                    MaybeConstructor::CallConstructor(property) => {
                        call_constructor = Some(property);
                    }
                    MaybeConstructor::CallAndConstructor(property) => {
                        constructor = Some(property.clone());
                        call_constructor = Some(property);
                    }
                }
            }
            ImplItem::Const(constant) => {
                let property = parse_constant(constant)?;

                match property {
                    MaybeStatic::Impl(property) => {
                        props.push(Prop::Constant(property));
                    }
                    MaybeStatic::Static(property) => {
                        static_props.push(Prop::Constant(property));
                    }
                }
            }
            _ => {}
        }
    }

    // Configuration for code generation:
    let config = crate::config::Config::new(Span::call_site());

    let init = init_props(props, &config);
    let (constructor_tokens, init_constructor) = init_constructor(
        &item_impl.self_ty,
        static_props,
        constructor,
        call_constructor,
        &config,
    )?;

    let try_into_value = &config.try_into_value;
    let object = &config.object;
    let object_handle = &config.object_handle;
    let value = &config.value;
    let error = &config.error;

    let init_fn = quote! {
        pub fn initialize_proto(mut obj: #object, func_proto: #value) -> ::core::result::Result<#object_handle, #error> {
            use yavashark_value::{AsAny, Obj, IntoValue, FromValue};
            use #try_into_value;

            #init

            let obj = obj.into_object();

            #init_constructor


            Ok(obj)
        }
    };

    // Append our generated initialization function to the impl block.
    item_impl.items.push(syn::parse2(init_fn)?);

    let tokens = quote! {
        #item_impl
        #constructor_tokens
    };

    Ok(tokens.into())
}

fn init_props(props: Vec<Prop>, config: &Config) -> TokenStream {
    // Generate initialization code from processed properties:
    let mut init = TokenStream::new();

    for prop in props {
        let (prop_tokens, name, js_name, prop_type) = match prop {
            Prop::Method(method) => (
                method.init_tokens(config),
                method.name,
                method.js_name,
                method.ty,
            ),
            Prop::Constant(constant) => (
                constant.init_tokens(config),
                constant.name,
                constant.js_name,
                Type::Normal,
            ),
        };

        let name = js_name
            .map(|js| quote! { #js })
            .unwrap_or_else(|| quote! { stringify!(#name) });

        let tokens = match prop_type {
            Type::Normal => {
                let variable = &config.variable;

                quote! {
                    {
                        let prop = #prop_tokens;
                        obj.define_variable(#name.into(), #variable::write_configig(prop.into()))?;
                    }
                }
            }
            Type::Get => {
                quote! {
                    {
                        let prop = #prop_tokens;
                        obj.define_getter(#name.into(), prop.into())?;
                    }
                }
            }
            Type::Set => {
                quote! {
                    {
                        let prop = #prop_tokens;
                        obj.define_setter(#name.into(), prop.into())?;
                    }
                }
            }
        };

        init.extend(tokens);
    }
    init
}

fn init_constructor(
    ty: &syn::Type,
    static_props: Vec<Prop>,
    constructor: Option<Method>,
    call_constructor: Option<Method>,
    config: &Config,
) -> syn::Result<(TokenStream, TokenStream)> {
    if static_props.is_empty() && constructor.is_none() && call_constructor.is_none() {
        return Ok((TokenStream::new(), TokenStream::new()));
    }


    let value = &config.value;
    let error = &config.error;
    let mut_obj = &config.mut_object;
    let object_handle = &config.object_handle;
    let realm = &config.realm;
    let try_into_value = &config.try_into_value;

    let name = ty_to_name(ty)?;
    let name = format_ident!("{}Constructor", name);
    let mut_name = format_ident!("Mutable{}", name);
    let args = match (constructor.is_some(), call_constructor.is_some()) {
        (true, true) => quote! { constructor, function },
        (true, false) => quote! { constructor },
        (false, true) => quote! { function },
        (false, false) => unreachable!(),
    };

    let mut constructor_tokens = quote! {
        #[yavashark_macro::object(#args)]
        #[derive(Debug)]
        pub struct #name {}
    };

    if let Some(constructor) = constructor {
        let fn_tok = constructor.init_tokes_direct(config, ty.to_token_stream());


        constructor_tokens.extend(quote! {
            impl yavashark_value::Constructor<#realm> for #name {
                fn construct(&self, realm: &mut #realm, mut args: std::vec::Vec<#value>) -> ::core::result::Result<#value, #error> {
                    use yavashark_value::{AsAny, Obj, IntoValue, FromValue};
                    use #try_into_value;
                    
                    #fn_tok
                }
            }
        });
    }

    if let Some(call_constructor) = call_constructor {
        let fn_tok = call_constructor.init_tokes_direct(config, ty.to_token_stream());

        constructor_tokens.extend(quote! {
            impl yavashark_value::Func<#realm> for #name {
                pub fn call(&self, realm: #realm, args: std::vec::Vec<#value>, this: #value) -> crate::Res<ObjectHandle> {
                    use yavashark_value::{AsAny, Obj, IntoValue, FromValue};
                    use #try_into_value;
                    
                    #fn_tok
                }
            }
        });
    }


    {
        let init = init_props(static_props, config);
        constructor_tokens.extend(quote! {
            impl #name {
                #[allow(clippy::new_ret_no_self)]
                pub fn new(func: & #value) -> ::core::result::Result<#object_handle, #error> {
                    use yavashark_value::Obj;
                    let mut this = Self {
                        inner: ::core::cell::RefCell::new(#mut_name {
                            object: #mut_obj::with_proto(func.copy()),
                        }),
                    };

                    this.initialize(func.copy())?;

                    Ok(this.into_object())
                }
                
                pub fn initialize(&mut self, func_proto: #value) -> core::result::Result<(), #error> {
                    use yavashark_value::{AsAny, Obj, IntoValue, FromValue};
                    use #try_into_value;

                    #init

                    Ok(())
                    
                }
            }
        });
    }

    let variable = &config.variable;

    let init_tokens = quote! {
        let constructor = #name::new(&func_proto)?;
        
        obj.define_variable("constructor".into(), constructor.clone().into())?;
        
        constructor.define_variable("prototype".into(), #variable::write_config(obj.clone().into()))?;
        
        
    };

    Ok((constructor_tokens, init_tokens))
}

fn ty_to_name(ty: &syn::Type) -> syn::Result<syn::Ident> {
    match ty {
        syn::Type::Path(path) => {
            let seg = path.path.segments.last().ok_or(syn::Error::new(
                path.span(),
                "Expected a type path with at least one segment",
            ))?;
            Ok(seg.ident.clone())
        }
        syn::Type::Reference(r) => ty_to_name(&r.elem),
        _ => Err(syn::Error::new(ty.span(), "Expected a type path")),
    }
}
