use crate::config::Config;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Expr, ImplItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// All methods and properties will be put on the prototype of the class => Array.prototype.map
    Prototype,
    /// All methods and properties will be put on the class itself => Math.random
    Raw,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Type {
    Normal,
    Get,
    Set,
}

#[allow(unused)]
pub fn properties(attrs: TokenStream1, item: TokenStream1) -> TokenStream1 {
    let mut mode = Mode::Prototype;
    let mut constructor = None;

    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("prototype") {
            mode = Mode::Prototype;
            Ok(())
        } else if meta.path.is_ident("raw") {
            mode = Mode::Raw;
            Ok(())
        } else if meta.path.is_ident("constructor") {
            meta.parse_nested_meta(|meta| {
                constructor = Some(meta.path);

                Ok(())
            });
            Ok(())
        } else {
            Err(meta.error("Unknown attribute"))
        }
    });

    syn::parse_macro_input!(attrs with attr_parser);

    let mut item: syn::ItemImpl = syn::parse_macro_input!(item);

    let mut props = Vec::new();

    for item in &mut item.items {
        match item {
            ImplItem::Fn(func) => {
                let mut js_name = None;
                let mut this = None;
                let mut realm = None;
                let mut variadic = None;
                let mut mode = mode;
                let mut has_receiver = false;
                let mut ty = Type::Normal;

                let mut args = Vec::new();

                func.sig.inputs.iter_mut().for_each(|arg| {
                    let pat = match arg {
                        syn::FnArg::Typed(pat) => pat,
                        syn::FnArg::Receiver(rec) => {
                            has_receiver = true;
                            return;
                        }
                    };

                    pat.attrs.retain_mut(|attr| {
                        if attr.path().is_ident("this") {
                            this = Some(args.len());
                            return false;
                        }

                        if attr.path().is_ident("realm") {
                            realm = Some(args.len());
                            return false;
                        }

                        if attr.path().is_ident("variadic") {
                            variadic = Some(args.len());
                            return false;
                        }

                        true
                    });

                    args.push((*pat.ty).clone());
                });

                let mut error = None;

                func.attrs.retain_mut(|attr| {
                    if attr.path().is_ident("prototype") {
                        mode = Mode::Prototype;
                        panic!("Mixed up modes currently not supported!")
                        // return false;
                    }

                    if attr.path().is_ident("raw") {
                        mode = Mode::Raw;
                        panic!("Mixed up modes currently not supported!")
                        // return false;
                    }

                    if attr.path().is_ident("prop") {
                        js_name = Some(attr.parse_args().unwrap());
                        return false;
                    }

                    if attr.path().is_ident("get") {
                        if ty == Type::Normal {
                            ty = Type::Get;
                        } else {
                            error = Some(syn::Error::new(
                                attr.span(),
                                "Cannot have both get and set on the same function",
                            ));
                        }

                        if let Err(e) = attr.meta.require_list() {
                            error = Some(e);
                        };

                        let name = match attr.parse_args() {
                            Ok(name) => name,
                            Err(e) => {
                                error = Some(e);
                                return false;
                            }
                        };

                        if js_name.is_some() {
                            error = Some(syn::Error::new(
                                attr.span(),
                                "Cannot have both prop and get on the same function",
                            ));
                        }

                        js_name = Some(name);
                        return false;
                    }

                    if attr.path().is_ident("set") {
                        if ty == Type::Normal {
                            ty = Type::Set;
                        } else {
                            error = Some(syn::Error::new(
                                attr.span(),
                                "Cannot have both get and set on the same function",
                            ));
                        }

                        if let Err(e) = attr.meta.require_list() {
                            error = Some(e);
                        };

                        let name = match attr.parse_args() {
                            Ok(name) => name,
                            Err(e) => {
                                error = Some(e);
                                return false;
                            }
                        };

                        if js_name.is_some() {
                            error = Some(syn::Error::new(
                                attr.span(),
                                "Cannot have both prop and get on the same function",
                            ));
                        }

                        js_name = Some(name);
                        return false;
                    }

                    true
                });

                if let Some(error) = error {
                    return error.to_compile_error().into();
                }

                props.push(Prop::Method(Method {
                    name: func.sig.ident.clone(),
                    js_name,
                    args,
                    this,
                    realm,
                    mode,
                    has_receiver,
                    ty,
                }))
            }

            ImplItem::Const(constant) => {
                let mut js_name = None;
                let mut mode = mode;

                constant.attrs.retain_mut(|attr| {
                    if attr.path().is_ident("prototype") {
                        mode = Mode::Prototype;
                        return false;
                    }

                    if attr.path().is_ident("raw") {
                        mode = Mode::Raw;
                        return false;
                    }

                    if attr.path().is_ident("prop") {
                        js_name = Some(attr.parse_args().unwrap());
                        return false;
                    }

                    true
                });

                props.push(Prop::Constant(Constant {
                    name: constant.ident.clone(),
                    js_name,
                    mode,
                }))
            }

            _ => {}
        }
    }

    let config = Config::new(Span::call_site());
    let variable = &config.variable;
    let object = &config.object;
    let value = &config.value;
    let handle = &config.object_handle;
    let error = &config.error;
    let try_into_value = &config.try_into_value;

    let mut init = TokenStream::new();

    for prop in props {
        let (prop_tokens, name, js_name, ty) = match prop {
            Prop::Method(method) => (
                method.init_tokens(&config),
                method.name,
                method.js_name,
                method.ty,
            ),
            Prop::Constant(constant) => (
                constant.init_tokens(&config),
                constant.name,
                constant.js_name,
                Type::Normal,
            ),
        };

        let name = js_name
            .map(|js_name| quote! {#js_name})
            .unwrap_or_else(|| quote! {stringify!(#name)});

        match ty {
            Type::Normal => {
                init.extend(quote! {
                    {
                        let prop = #prop_tokens;

                        obj.define_variable(#name.into(), #variable::new(prop.into()))?;
                    }
                });
            }
            Type::Get => {
                init.extend(quote! {
                    {
                        let prop = #prop_tokens;

                        obj.define_getter(#name.into(), prop.into())?;
                    }
                });
            }

            Type::Set => {
                init.extend(quote! {
                    {
                        let prop = #prop_tokens;

                        obj.define_setter(#name.into(), prop.into())?;
                    }
                });
            }
        }
    }

    let (constructor, proto_define) = if let Some(constructor) = constructor {
        (
            quote! {
                let constructor = #constructor(&obj, &func_proto)?;
                obj.define_variable("constructor".into(), #variable::new(constructor.clone().into()))?;

            },
            quote! {
                constructor.define_variable("prototype".into(), #variable::new(obj.clone().into()))?;
            },
        )
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    let init_fn = match mode {
        Mode::Prototype => quote! {
            pub fn initialize_proto(mut obj: #object, func_proto: #value) -> Result<#handle, #error> {
                use yavashark_value::{AsAny, Obj, IntoValue, FromValue};
                use #try_into_value;

                #init

                #constructor

                let obj = obj.into_object();

                #proto_define


                Ok(obj)
            }
        },
        Mode::Raw => quote! {
            pub fn initialize(&mut self, func_proto: #value) -> Result<(), #error> {
                use yavashark_value::{AsAny, Obj, IntoValue, FromValue};
                use #try_into_value;

                let obj = self;

                #init

                Ok(())
            }
        },
    };

    item.items.push(ImplItem::Verbatim(init_fn));

    item.to_token_stream().into()
}

enum Prop {
    Method(Method),
    Constant(Constant),
}

struct Method {
    name: syn::Ident,
    js_name: Option<Expr>,
    args: Vec<syn::Type>,
    this: Option<usize>,
    realm: Option<usize>,
    #[allow(unused)]
    mode: Mode,
    has_receiver: bool,
    ty: Type,
}

#[allow(unused)]
struct Constant {
    name: syn::Ident,
    js_name: Option<Expr>,
    mode: Mode,
}

impl Method {
    fn init_tokens(&self, config: &Config) -> TokenStream {
        let native_function = &config.native_function;

        let name = &self.name;
        let extractor = &config.extractor;
        let extract_value = &config.extract_value;

        let mut arg_prepare = quote! {
            let mut extractor = #extractor::new(&mut args);
        };
        let mut call_args = TokenStream::new();

        for (i, ty) in self.args.iter().enumerate() {
            let argname = syn::Ident::new(&format!("arg{}", i), Span::call_site());

            if Some(i) == self.this {
                arg_prepare.extend(quote! {
                    let #argname = this.copy();
                });
            } else if Some(i) == self.realm {
                arg_prepare.extend(quote! {
                    let #argname = realm;
                });
            } else {
                arg_prepare.extend(quote! {
                    let #argname = #extract_value::<#ty>::extract(&mut extractor)?;
                });
            }

            let refs = if matches!(ty, syn::Type::Reference(_)) && Some(i) != self.realm {
                quote! {&}
            } else {
                TokenStream::new()
            };

            call_args.extend(quote! {
                #refs #argname,
            });
        }

        let call = if self.has_receiver {
            quote! {
                this.#name(#call_args)
            }
        } else {
            quote! {
                Self::#name(#call_args)
            }
        };

        let prepare_receiver = if self.has_receiver {
            quote! {
                let this: yavashark_garbage::OwningGcGuard<_, Self> = FromValue::from_value(this)?;
            }
        } else {
            TokenStream::new()
        };

        let name = self
            .js_name
            .clone()
            .map(|js_name| quote! {#js_name})
            .unwrap_or_else(|| quote! {#name});

        quote! {
            #native_function::with_proto(stringify!(#name), |mut args, mut this, realm| {
                #arg_prepare
                #prepare_receiver
                #call.try_into_value()
            }, func_proto.copy())
        }
    }
}

impl Constant {
    fn init_tokens(&self, _config: &Config) -> TokenStream {
        let name = &self.name;

        quote! {
            Self::#name.into_value()
        }
    }
}
