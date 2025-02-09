use crate::config::Config;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::parse::Parse;
use syn::{Expr, ImplItem};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Mode {
    /// All methods and properties will be put on the prototype of the class => Array.prototype.map
    Prototype,
    /// All methods and properties will be put on the class itself => Math.random
    Raw,
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

                let mut args = 0;

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
                            this = Some(args);
                            return false;
                        }

                        if attr.path().is_ident("realm") {
                            realm = Some(args);
                            return false;
                        }

                        if attr.path().is_ident("variadic") {
                            variadic = Some(args);
                            return false;
                        }

                        true
                    });

                    args += 1;
                });

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

                    true
                });

                props.push(Prop::Method(Method {
                    name: func.sig.ident.clone(),
                    js_name,
                    args,
                    this,
                    realm,
                    variadic,
                    mode,
                    has_receiver,
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
        let (prop, name, js_name) = match prop {
            Prop::Method(method) => (method.init_tokens(&config), method.name, method.js_name),
            Prop::Constant(constant) => (
                constant.init_tokens(&config),
                constant.name,
                constant.js_name,
            ),
        };

        let name = js_name
            .map(|js_name| quote! {#js_name})
            .unwrap_or_else(|| quote! {stringify!(#name)});

        init.extend(quote! {
            {
                let prop = #prop;

                obj.define_variable(#name.into(), #variable::new(prop.into()))?;
            }
        });
    }

    let constructor = if let Some(constructor) = constructor {
        quote! {
            let constructor = #constructor(&obj, &func_proto)?;
            obj.define_variable("constructor".into(), #variable::new(constructor.into()))?;
        }
    } else {
        TokenStream::new()
    };

    let init_fn = match mode {
        Mode::Prototype => quote! {
            pub fn initialize_proto(mut obj: #object, func_proto: #value) -> Result<#handle, #error> {
                use yavashark_value::{AsAny, Obj, IntoValue, FromValue};
                use #try_into_value;

                #init

                #constructor

                let obj = obj.into_object();


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
    args: usize,
    this: Option<usize>,
    realm: Option<usize>,
    variadic: Option<usize>,
    #[allow(unused)]
    mode: Mode,
    has_receiver: bool,
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
        let error = &config.error;

        let mut arg_prepare = TokenStream::new();
        let mut call_args = TokenStream::new();

        for i in 0..self.args {
            let argname = syn::Ident::new(&format!("arg{}", i), Span::call_site());

            if Some(i) == self.this {
                arg_prepare.extend(quote! {
                    let #argname = this.copy();
                });
            } else if Some(i) == self.realm {
                arg_prepare.extend(quote! {
                    let #argname = realm;
                });
            } else if Some(i) == self.variadic {
                arg_prepare.extend(quote! {
                    let #argname = args.get(#i..).unwrap_or_default();
                });
            } else {
                arg_prepare.extend(quote! {
                    let #argname = FromValue::from_value(args.get(#i).ok_or_else(|| #error::new("Missing argument"))?.copy())?;
                });
            }

            call_args.extend(quote! {
                #argname,
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
            .unwrap_or_else(|| quote! {stringify!(#name)});

        quote! {
            #native_function::with_proto(stringify!(#name), |args, mut this, realm| {
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
