use crate::properties_new::{MaybeConstructor, Type};
use proc_macro2::{Span, TokenStream};
use quote::quote;
use syn::spanned::Spanned;
use syn::Expr;

#[derive(Clone)]
pub struct Method {
    pub name: syn::Ident,
    pub js_name: Option<Expr>,
    pub args: Vec<syn::Type>,
    pub this: Option<usize>,
    pub realm: Option<usize>,
    pub has_receiver: bool,
    pub ty: Type,
}

impl Method {
    pub fn init_tokens(&self, config: &crate::config::Config) -> TokenStream {
        let self_ty = quote! { Self };

        self.init_tokens_self(config, self_ty)
    }



    pub fn init_tokens_self(&self, config: &crate::config::Config, self_ty: TokenStream) -> TokenStream {
        let native_function = &config.native_function;
        let name_ident = &self.name;
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
                quote! { & }
            } else {
                TokenStream::new()
            };
            call_args.extend(quote! {
                #refs #argname,
            });
        }

        let call = if self.has_receiver {
            quote! {
                this.#name_ident(#call_args)
            }
        } else {
            quote! {
                #self_ty::#name_ident(#call_args)
            }
        };

        let prepare_receiver = if self.has_receiver {
            quote! {
                let this: yavashark_garbage::OwningGcGuard<_, Self> = FromValue::from_value(this)?;
            }
        } else {
            TokenStream::new()
        };

        let js_name = self
            .js_name
            .clone()
            .map(|js| quote! { #js })
            .unwrap_or_else(|| quote! { stringify!(#name_ident) });
        let length = self.args.len();

        quote! {
            #native_function::with_proto_and_len(#js_name.as_ref(), |mut args, mut this, realm| {
                #arg_prepare
                #prepare_receiver
                #call.try_into_value()
            }, func_proto.copy(), #length)
        }
    }

    pub fn init_tokes_direct(
        &self,
        config: &crate::config::Config,
        self_ty: TokenStream,
    ) -> TokenStream {
        let name_ident = &self.name;
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
                quote! { & }
            } else {
                TokenStream::new()
            };
            call_args.extend(quote! {
                #refs #argname,
            });
        }

        let call = if self.has_receiver {
            quote! {
                self.#name_ident(#call_args)
            }
        } else {
            quote! {
                #self_ty::#name_ident(#call_args)
            }
        };

        let prepare_receiver = if self.has_receiver {
            quote! {
                let this: yavashark_garbage::OwningGcGuard<_, Self> = FromValue::from_value(this)?;
            }
        } else {
            TokenStream::new()
        };

        quote! {
                #arg_prepare
                #prepare_receiver
                #call.try_into_value()
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum MethodType {
    Impl,
    Static,
    Constructor,
    CallConstructor,
    CallAndConstructor,
}

pub fn parse_method(
    func: &mut syn::ImplItemFn,
) -> Result<(MaybeConstructor<Method>, bool), syn::Error> {
    let mut js_name = None;
    let mut this = None;
    let mut realm = None;
    let mut has_receiver = false;
    let mut ty = Type::Normal;

    let mut maybe_static = MethodType::Static;

    let mut args = Vec::new();
    for arg in func.sig.inputs.iter_mut() {
        match arg {
            syn::FnArg::Receiver(_) => {
                has_receiver = true;
                maybe_static = MethodType::Impl;
            }
            syn::FnArg::Typed(pat) => {
                pat.attrs.retain_mut(|attr| {
                    if attr.path().is_ident("this") {
                        this = Some(args.len());
                        return false;
                    } else if attr.path().is_ident("realm") {
                        realm = Some(args.len());
                        return false;
                    }
                    
                    true
                });
                args.push((*pat.ty).clone());
            }
        }
    }

    // Process the function level attributes:
    let mut encountered_error = None;
    func.attrs.retain_mut(|attr| {
        if attr.path().is_ident("prop") {
            let n = match attr.parse_args()
                .map_err(|e| syn::Error::new(e.span(), e)) {
                Ok(n) => n,
                Err(e) => {
                    encountered_error = Some(e);
                    return false;
                }
            };


            js_name = Some(n);
            return false;
        } else if attr.path().is_ident("get") {
            if ty != Type::Normal {
                encountered_error = Some(syn::Error::new(
                    attr.span(),
                    "Cannot have both get and set on the same function",
                ));
            }
            attr.parse_args::<Expr>()
                .map(|name| {
                    if js_name.is_some() {
                        encountered_error = Some(syn::Error::new(
                            attr.span(),
                            "Cannot have both prop and get on the same function",
                        ));
                    }
                    js_name = Some(name);
                })
                .map_err(|e| encountered_error = Some(e))
                .ok();
            ty = Type::Get;

            return false;
        } else if attr.path().is_ident("set") {
            if ty != Type::Normal {
                encountered_error = Some(syn::Error::new(
                    attr.span(),
                    "Cannot have both get and set on the same function",
                ));
            }
            attr.parse_args::<Expr>()
                .map(|name| {
                    if js_name.is_some() {
                        encountered_error = Some(syn::Error::new(
                            attr.span(),
                            "Cannot have both prop and set on the same function",
                        ));
                    }
                    js_name = Some(name);
                })
                .map_err(|e| encountered_error = Some(e))
                .ok();
            ty = Type::Set;

            return false;
        } else if attr.path().is_ident("static") {
            maybe_static = MethodType::Static;

            return false;
        } else if attr.path().is_ident("nonstatic") {
            maybe_static = MethodType::Impl;

            return false;
        } else if attr.path().is_ident("constructor") {
            if maybe_static == MethodType::CallConstructor {
                maybe_static = MethodType::CallAndConstructor;
            } else {
                maybe_static = MethodType::Constructor;
            }

            return false;
        } else if attr.path().is_ident("call_constructor") {
            if maybe_static == MethodType::Constructor {
                maybe_static = MethodType::CallAndConstructor;
            } else {
                maybe_static = MethodType::CallConstructor;
            }

            return false;
        }

        true
    });

    if let Some(err) = encountered_error {
        return Err(err);
    }

    let is_constructor = matches!(
        maybe_static,
        MethodType::Constructor | MethodType::CallConstructor | MethodType::CallAndConstructor
    );

    if ty != Type::Normal && is_constructor {
        return Err(syn::Error::new(
            func.sig.ident.span(),
            "Getters and setters must be on non constructor methods",
        ));
    }

    if this.is_some() && matches!(maybe_static, MethodType::Constructor | MethodType::CallAndConstructor) {
        return Err(syn::Error::new(
            func.sig.ident.span(),
            "Cannot have this on constructor methods",
        ));
    }

    Ok((
        match maybe_static {
            MethodType::Impl => MaybeConstructor::Impl,
            MethodType::Static => MaybeConstructor::Static,
            MethodType::Constructor => MaybeConstructor::Constructor,
            MethodType::CallConstructor => MaybeConstructor::CallConstructor,
            MethodType::CallAndConstructor => MaybeConstructor::CallAndConstructor,
        }(Method {
            name: func.sig.ident.clone(),
            js_name,
            args,
            this,
            realm,
            has_receiver,
            ty,
        }),
        has_receiver,
    ))
}
