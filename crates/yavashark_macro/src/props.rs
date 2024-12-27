use crate::config::Config;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ImplItem;

#[derive(Debug, Clone, Copy)]
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
                let mut js_name = func.sig.ident.clone();
                let mut this = None;
                let mut realm = None;
                let mut variadic = None;
                let mut mode = mode;
                let mut has_receiver = false;

                func.sig.inputs.iter().fold(0, |idx, arg| {
                    let syn::FnArg::Typed(pat) = arg else {
                        has_receiver = true;
                        return idx;
                    };

                    pat.attrs.iter().for_each(|attr| {
                        if attr.path().is_ident("this") {
                            this = Some(idx);
                        }

                        if attr.path().is_ident("realm") {
                            realm = Some(idx);
                        }

                        if attr.path().is_ident("rest") {
                            variadic = Some(idx);
                        }
                    });

                    idx + 1
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
                }))
            }

            ImplItem::Const(constant) => {
                let mut js_name = constant.ident.clone();
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

    TokenStream::new().into()
}

enum Prop {
    Method(Method),
    Constant(Constant),
}

struct Method {
    name: syn::Ident,
    js_name: syn::Ident,
    args: usize,
    this: Option<usize>,
    realm: Option<usize>,
    variadic: Option<usize>,
    mode: Mode,
    has_receiver: bool,
}

struct Constant {
    name: syn::Ident,
    js_name: syn::Ident,
    mode: Mode,
}

impl Method {
    fn init_tokens(&self, config: &Config) -> TokenStream {
        let native_function = &config.native_function;

        let name = &self.name;

        let js_name = &self.js_name;
        
        
        let mut arg_prepare = TokenStream::new();
        let mut call_args = TokenStream::new();
        
        for i in 0..self.args {
            let argname = syn::Ident::new(&format!("arg{}", i), proc_macro2::Span::call_site());
            
            if Some(i) == self.this {
                arg_prepare.extend(quote! {
                    let #argname = this.copy();
                });
                
                continue;
            }
            
            if Some(i) == self.realm {
                arg_prepare.extend(quote! {
                    let #argname = realm;
                });
                
                continue;
            }
            
            if Some(i) == self.variadic {
                todo!()
            }
            
            arg_prepare.extend(quote! {
                let #argname = args.get(#i).ok_or_else(|| Error::new("Missing argument"))?;
            });
            
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
        
        quote! {
            #native_function::with_proto(stringify!(#js_name), |args, mut this, realm| {
                #arg_prepare
                #call.into_value().into();

            });
        }
    }
}
