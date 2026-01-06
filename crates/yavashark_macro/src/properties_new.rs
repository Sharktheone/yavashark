mod constant;
mod method;

use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Ident, Span, TokenStream};
use quote::{format_ident, quote, ToTokens};
use syn::{spanned::Spanned, Expr, ImplItem, ItemImpl, Path};

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
    Both(T),
}

#[derive(Default, FromMeta)]
pub struct PropertiesArgs {
    extends: Option<Ident>,
    #[allow(unused)]
    or: Option<Expr>,
    override_object: Option<Path>,
    to_string_tag: Option<String>,
    intrinsic_name: Option<Ident>,
    constructor_name: Option<String>,
    #[darling(default)]
    no_intrinsic: bool,
    #[darling(default)]
    no_partial: bool,
    #[darling(default)]
    raw: bool,
}

#[allow(unused)]
pub fn properties(attrs: TokenStream1, item: TokenStream1) -> syn::Result<TokenStream1> {
    let mut item_impl = syn::parse::<ItemImpl>(item)?;

    let attr_args = match NestedMeta::parse_meta_list(attrs.clone().into()) {
        Ok(v) => v,
        Err(e) => {
            return Err(e);
        }
    };

    let args = match PropertiesArgs::from_list(&attr_args) {
        Ok(args) => args,
        Err(e) => return Err(e.into()),
    };

    if args.no_intrinsic && args.intrinsic_name.is_some() {
        return Err(syn::Error::new(
            Span::call_site(),
            "Cannot specify intrinsic_name when no_intrinsic is set",
        ));
    }

    if !args.no_intrinsic && args.intrinsic_name.is_none() {
        return Err(syn::Error::new(
            Span::call_site(),
            "Must specify intrinsic_name unless no_intrinsic is set",
        ));
    }

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
                        if args.raw {
                            return Err(syn::Error::new(
                                constant.span(),
                                "Cannot use 'impl' with 'raw' option",
                            ));
                        }

                        props.push(Prop::Constant(property));
                    }
                    MaybeStatic::Static(property) => {
                        static_props.push(Prop::Constant(property));
                    }
                    MaybeStatic::Both(property) => {
                        if args.raw {
                            return Err(syn::Error::new(
                                constant.span(),
                                "Cannot use 'both' with 'raw' option",
                            ));
                        }
                        props.push(Prop::Constant(property.clone()));
                        static_props.push(Prop::Constant(property));
                    }
                }
            }
            _ => {}
        }
    }

    let config = Config::new(Span::call_site());

    let mut init = init_props(props, &config, None);

    if let Some(tag) = args.to_string_tag {
        let value = &config.value;
        let variable = &config.variable;
        let env = &config.env_path;
        let realm = &config.realm;
        let symbol = &config.symbol;

        init.extend(quote! {
            {
                let to_string_tag = #value::from(#tag);
                obj.define_property_attributes(
                    #symbol::TO_STRING_TAG.into(),
                    #variable::config(to_string_tag),
                    realm,
                )?;
            }
        });
    }

    let (constructor_tokens, init_constructor) = init_constructor(
        &item_impl.self_ty,
        static_props,
        constructor,
        call_constructor,
        &config,
        args.extends.is_some(),
        args.constructor_name.as_deref(),
    )?;

    let try_into_value = &config.try_into_value;
    let proto_object = args.override_object.as_ref().unwrap_or(&config.object);
    let object_handle = &config.object_handle;
    let value = &config.value;
    let error = &config.error;
    let env = &config.env_path;
    let realm = &config.realm;
    let intrinsic = &config.intrinsic;
    let res = &config.res;
    let obj = &config.object;

    let intrinsic_get = if let Some(name) = args.intrinsic_name.as_ref() {
        if args.no_partial {
            quote! {
                Ok(realm.intrinsics.clone_public().#name.clone())
            }
        } else {
            quote! {
                Ok(realm.intrinsics.clone_public().#name.get(realm)?.clone())
            }
        }
    } else {
        quote! {
            Self::initialize(realm)
        }
    };

    let struct_name = &item_impl.self_ty;

    let get_prototype = if let Some(extends) = args.extends {
        quote! {
            #extends::get_intrinsic(realm)?
        }
    } else {
        quote! {
            realm.intrinsics.obj.clone()
        }
    };

    let tokens = quote! {
        #item_impl
        #constructor_tokens

        impl #intrinsic for #struct_name {
            fn initialize(realm: &mut #realm) -> #res<#object_handle> {
                use #env::value::{Obj, IntoValue, FromValue};
                use #try_into_value;
                let mut obj = #obj::raw_with_proto(#get_prototype);


                #init

                let obj = obj.into_object();

                #init_constructor


                Ok(obj)
            }

            fn get_intrinsic(realm: &mut #realm) -> #res<#object_handle> {
                #intrinsic_get
            }

            fn get_global(realm: &mut Realm) -> #res<#object_handle> {
                let this = Self::get_intrinsic(realm)?;

                this.get("constructor", realm)?
                    .to_object()
            }
        }
    };

    Ok(tokens.into())
}

fn init_props(props: Vec<Prop>, config: &Config, self_ty: Option<TokenStream>) -> TokenStream {
    let mut init = TokenStream::new();
    let self_ty = self_ty.unwrap_or_else(|| quote! { Self });
    let variable = &config.variable;
    let attributes = &config.attributes;

    for prop in props {
        let (prop_tokens, name, js_name, prop_type, var_create) = match prop {
            Prop::Method(method) => (
                method.init_tokens_self(config, self_ty.clone()),
                method.name,
                method.js_name,
                method.ty,
                quote! {#variable::write_config(prop.into())},
            ),
            Prop::Constant(constant) => {
                let writable = constant.writable;
                let enumerable = constant.enumerable;
                let configurable = constant.configurable;

                let variable_fn = quote! { #variable::new_with_attributes(prop.into(), #writable, #enumerable, #configurable) };

                (
                    constant.init_tokens(config, self_ty.clone()),
                    constant.name,
                    constant.js_name,
                    Type::Normal,
                    variable_fn,
                )
            }
        };

        let name = js_name
            .map(|js| quote! { #js })
            .unwrap_or_else(|| quote! { stringify!(#name) });

        let tokens = match prop_type {
            Type::Normal => {
                quote! {
                    {
                        let prop = #prop_tokens;

                        obj.define_property_attributes(#name.into(), #var_create, realm)?;
                    }
                }
            }
            Type::Get => {
                quote! {
                    {
                        let prop = #prop_tokens;
                        obj.define_getter_attributes(#name.into(), prop.into(), #attributes::config(), realm)?;
                    }
                }
            }
            Type::Set => {
                quote! {
                    {
                        let prop = #prop_tokens;
                        obj.define_setter_attributes(#name.into(), prop.into(), #attributes::config(), realm)?;
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
    extends: bool,
    constructor_name: Option<&str>,
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
    let env = &config.env_path;

    let ty_name = ty_to_name(ty)?;
    let name = format_ident!("{}Constructor", ty_name);
    let mut_name = format_ident!("Mutable{}", name);
    let args = match (constructor.is_some(), call_constructor.is_some()) {
        (true, true) => quote! { constructor, function },
        (true, false) => quote! { constructor },
        (false, true) => quote! { function },
        (false, false) => quote! {},
    };

    let mut constructor_tokens = quote! {
        #[yavashark_macro::object(#args)]
        #[derive(Debug)]
        pub struct #name {}
    };

    let mut constructor_length = 0;

    if let Some(ref constructor) = constructor {
        let fn_tok = constructor.init_tokes_direct(config, ty.to_token_stream());

        constructor_tokens.extend(quote! {
            impl #env::value::Constructor for #name {
                fn construct(&self, realm: &mut #realm, mut args: std::vec::Vec<#value>) -> ::core::result::Result<#object_handle, #error> {
                    use #env::value::{Obj, IntoValue, FromValue};
                    use #try_into_value;

                    #fn_tok?.to_object()
                }
            }
        });

        constructor_length = constructor.calculate_length().0;
    }

    if let Some(ref call_constructor) = call_constructor {
        let fn_tok = call_constructor.init_tokes_direct(config, ty.to_token_stream());

        constructor_tokens.extend(quote! {
            impl #env::value::Func for #name {
                fn call(&self, realm: &mut #realm, mut args: std::vec::Vec<#value>, this: #value) -> crate::ValueResult {
                    use #env::value::{Obj, IntoValue, FromValue};
                    use #try_into_value;

                    #fn_tok
                }
            }
        });

        if constructor.is_none() || constructor.as_ref().is_some_and(|c| c.length.is_none()) {
            constructor_length = call_constructor
                .calculate_length()
                .0
                .max(constructor_length);
        }
    }

    {
        let init = init_props(static_props, config, Some(ty.to_token_stream()));
        constructor_tokens.extend(quote! {
            impl #name {
                #[allow(clippy::new_ret_no_self)]
                pub fn new(proto: #object_handle, realm: &mut #realm) -> ::core::result::Result<#object_handle, #error> {
                    use #env::value::Obj;
                    let mut this = Self {
                        inner: ::core::cell::RefCell::new(#mut_name {
                            object: #mut_obj::with_proto(proto),
                        }),
                    };

                    this.initialize(realm)?;

                    Ok(this.into_object())
                }

                pub fn initialize(&mut self, realm: &mut #realm) -> core::result::Result<(), #error> {
                    use #env::value::{Obj, IntoValue, FromValue};
                    use #try_into_value;
                    let obj = self;

                    #init

                    Ok(())
                }
            }
        });
    }

    let variable = &config.variable;

    let constr_proto = if extends {
        quote! { {
            obj.prototype(realm)?.to_object()?.resolve_property("constructor", realm)?.unwrap_or(Value::Undefined).to_object()?
        } }
    } else {
        quote! { realm.intrinsics.func.clone() }
    };

    let display_name = if let Some(custom_name) = constructor_name {
        quote! { #custom_name }
    } else {
        quote! { stringify!(#ty_name) }
    };

    let init_tokens = quote! {
        let constructor = #name::new(#constr_proto, realm)?;

        obj.define_property_attributes("constructor".into(), #variable::write_config(constructor.clone().into()), realm)?;

        constructor.define_property_attributes("prototype".into(), #variable::new_read_only(obj.clone().into()), realm)?;
        constructor.define_property_attributes("length".into(), #variable::config(#value::from(#constructor_length)).into(), realm)?;
        constructor.define_property_attributes("name".into(), #variable::config(#value::from(#display_name).into()), realm)?;


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
