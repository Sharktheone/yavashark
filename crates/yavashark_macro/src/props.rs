use crate::config::Config;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::ImplItem;

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

    let attr_parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("prototype") {
            mode = Mode::Prototype;
            Ok(())
        } else if meta.path.is_ident("raw") {
            mode = Mode::Raw;
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
                let mut rec_mutability = false;

                func.sig
                    .inputs
                    .iter_mut()
                    .enumerate()
                    .for_each(|(idx, arg)| {
                        let pat = match arg {
                            syn::FnArg::Typed(pat) => pat,
                            syn::FnArg::Receiver(rec) => {
                                has_receiver = true;
                                rec_mutability = rec.mutability.is_some();
                                return;
                            }
                        };

                        let mut remove = Vec::new();

                        pat.attrs.iter().enumerate().for_each(|(idx_remove, attr)| {
                            if attr.path().is_ident("this") {
                                this = Some(idx);
                                remove.push(idx_remove);
                            }

                            if attr.path().is_ident("realm") {
                                realm = Some(idx);
                                remove.push(idx_remove);
                            }

                            if attr.path().is_ident("variadic") {
                                variadic = Some(idx);
                                remove.push(idx_remove);
                            }
                        });

                        remove.sort();

                        for idx in remove.into_iter().rev() {
                            pat.attrs.remove(idx);
                        }
                    });

                func.attrs.iter().for_each(|attr| {
                    if attr.path().is_ident("prototype") {
                        mode = Mode::Prototype;
                        panic!("Mixed up modes currently not supported!")
                    }

                    if attr.path().is_ident("raw") {
                        mode = Mode::Raw;
                        panic!("Mixed up modes currently not supported!")
                    }
                });

                props.push(Prop::Method(Method {
                    name: func.sig.ident.clone(),
                    js_name,
                    args: func.sig.inputs.len(),
                    this,
                    realm,
                    variadic,
                    mode,
                    has_receiver,
                    rec_mutability,
                }))
            }

            ImplItem::Const(constant) => {
                let mut js_name = None;
                let mut mode = mode;

                constant.attrs.iter().for_each(|attr| {
                    if attr.path().is_ident("prototype") {
                        mode = Mode::Prototype;
                    }

                    if attr.path().is_ident("raw") {
                        mode = Mode::Raw;
                    }
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

                obj.define_variable(#name.into(), #variable::new(prop.into()));
            }
        });
    }

    let init_fn = match mode {
        Mode::Prototype => quote! {
            fn initialize_proto(mut obj: #object, func_proto: #value) -> Result<#handle, #error> {
                use yavashark_value::{AsAny, Obj, IntoValue, FromValue};
                use #try_into_value;

                #init

                let obj = obj.into_object();


                Ok(obj)
            }
        },
        Mode::Raw => quote! {
            fn initialize(&mut self, func_proto: #value) -> Result<(), #error> {
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
    js_name: Option<syn::Path>,
    args: usize,
    this: Option<usize>,
    realm: Option<usize>,
    variadic: Option<usize>,
    #[allow(unused)]
    mode: Mode,
    has_receiver: bool,
    rec_mutability: bool,
}

#[allow(unused)]
struct Constant {
    name: syn::Ident,
    js_name: Option<syn::Path>,
    mode: Mode,
}

impl Method {
    fn init_tokens(&self, config: &Config) -> TokenStream {
        let native_function = &config.native_function;

        let name = &self.name;

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
                    let #argname = FromValue::from_value(args.get(#i).ok_or_else(|| Error::new("Missing argument"))?.copy())?;
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
            if self.rec_mutability {
                quote! {
                    let mut this: yavashark_garbage::collectable::OwningGcMutRefCellGuard<_, Self> = FromValue::from_value(this)?;
                }
            } else {
                quote! {
                    let this: yavashark_garbage::collectable::OwningGcRefCellGuard<_, Self> = FromValue::from_value(this)?;
                }
            }
        } else {
            TokenStream::new()
        };

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
