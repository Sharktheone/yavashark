use crate::config::Config;
use crate::deref_type;
use proc_macro::TokenStream as TokenStream1;
use proc_macro2::{Span, TokenStream};
use quote::{quote, ToTokens};
use syn::spanned::Spanned;
use syn::{Expr, ImplItem, Path};

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
    let mut proto_default = None;
    let mut no_intrinsic = false;
    let mut intrinsic_name = None;

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
        } else if meta.path.is_ident("default") {
            meta.parse_nested_meta(|meta| {
                proto_default = Some((meta.path, false));

                Ok(())
            });

            Ok(())
        } else if meta.path.is_ident("default_null") {
            meta.parse_nested_meta(|meta| {
                proto_default = Some((meta.path, true));

                Ok(())
            });

            Ok(())
        } else if meta.path.is_ident("no_intrinsic") {
            no_intrinsic = true;
            Ok(())
        } else if meta.path.is_ident("intrinsic_name") {
            meta.parse_nested_meta(|meta| {
                intrinsic_name = Some(meta.path);

                Ok(())
            });
            Ok(())
        } else {
            Err(meta.error("Unknown attribute"))
        }
    });

    syn::parse_macro_input!(attrs with attr_parser);

    if no_intrinsic && intrinsic_name.is_some() {
        return syn::Error::new(
            Span::call_site(),
            "Cannot have both no_intrinsic and intrinsic_name",
        )
        .to_compile_error()
        .into();
    }

    if !no_intrinsic && intrinsic_name.is_none() && mode == Mode::Prototype {
        return syn::Error::new(
            Span::call_site(),
            "Must have either no_intrinsic or intrinsic_name",
        )
        .to_compile_error()
        .into();
    }

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
                let mut writable = false;

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

                    if attr.path().is_ident("writable") {
                        writable = true;
                        return false;
                    }

                    true
                });

                props.push(Prop::Constant(Constant {
                    name: constant.ident.clone(),
                    js_name,
                    mode,
                    writable,
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
    let env = &config.env_path;
    let realm = &config.realm;

    let mut init = TokenStream::new();

    for prop in props {
        let (prop_tokens, name, js_name, ty, variable_fn) = match prop {
            Prop::Method(method) => (
                method.init_tokens(&config, proto_default.as_ref()),
                method.name,
                method.js_name,
                method.ty,
                quote! {#variable::write_config},
            ),
            Prop::Constant(constant) => {
                let variable_fn = if constant.writable {
                    quote! {#variable::write}
                } else {
                    quote! {#variable::new_read_only}
                };

                (
                    constant.init_tokens(&config),
                    constant.name,
                    constant.js_name,
                    Type::Normal,
                    variable_fn,
                )
            }
        };

        let name = js_name
            .map(|js_name| quote! {#js_name})
            .unwrap_or_else(|| quote! {stringify!(#name)});

        match ty {
            Type::Normal => {
                init.extend(quote! {
                    {
                        let prop = #prop_tokens;

                        obj.define_property_attributes(#name.into(), #variable_fn(prop.into()), realm)?;
                    }
                });
            }
            Type::Get => {
                init.extend(quote! {
                    {
                        let prop = #prop_tokens;

                        obj.define_getter(#name.into(), prop.into(), realm)?;
                    }
                });
            }

            Type::Set => {
                init.extend(quote! {
                    {
                        let prop = #prop_tokens;

                        obj.define_setter(#name.into(), prop.into(), realm)?;
                    }
                });
            }
        }
    }

    let (constructor, proto_define) = if let Some(constructor) = constructor {
        (
            quote! {
                let constructor = #constructor(&obj, realm.intrinsics.func.clone(), realm)?;
                obj.define_property_attributes("constructor".into(), #variable::write_config(constructor.clone().into()), realm)?;

            },
            quote! {
                constructor.define_property_attributes("prototype".into(), #variable::new_read_only(obj.clone().into()), realm)?;
            },
        )
    } else {
        (TokenStream::new(), TokenStream::new())
    };

    match mode {
        Mode::Prototype => {
            let intrinsic = &config.intrinsic;
            let struct_name = &item.self_ty;
            let obj = &config.object;
            let res = &config.res;
            let object_handle = &config.object_handle;

            let intrinsic_get = if let Some(name) = intrinsic_name.as_ref() {
                quote! {
                    Ok(realm.intrinsics.clone_public().#name.get(realm)?.clone())
                }
            } else {
                quote! {
                    Self::initialize(realm)
                }
            };

            let init = quote! {
                impl #intrinsic for #struct_name {
                    fn initialize(realm: &mut #realm) -> Result<#handle, #error> {
                        use #env::value::{Obj, IntoValue, FromValue};
                        use #try_into_value;

                        let mut obj = #obj::raw_with_proto(
                            realm.intrinsics.obj.clone(),
                        );

                        #init

                        #constructor

                        let obj = obj.into_object();

                        #proto_define


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


                // pub fn initialize_proto(mut obj: #object, func_proto: #handle, realm: &mut #realm) -> Result<#handle, #error> {
                //     use #env::value::{Obj, IntoValue, FromValue};
                //     use #try_into_value;
                //
                //     #init
                //
                //     #constructor
                //
                //     let obj = obj.into_object();
                //
                //     #proto_define
                //
                //
                //     Ok(obj)
                // }
            };

            quote! {
                #item

                #init
            }
        }
        Mode::Raw => {
            let init_fn = quote! {
                pub fn initialize(&mut self, realm: &mut #realm) -> Result<(), #error> {
                    use #env::value::{Obj, IntoValue, FromValue};
                    use #try_into_value;

                    let obj = self;

                    #init

                    Ok(())
                }
            };

            item.items.push(ImplItem::Verbatim(init_fn));

            item.to_token_stream()
        }
    }
    .into()
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
    writable: bool,
}

impl Method {
    fn init_tokens(&self, config: &Config, proto_default: Option<&(Path, bool)>) -> TokenStream {
        let native_function = &config.native_function;

        let name = &self.name;
        let extractor = &config.extractor;
        let extract_value = &config.extract_value;

        let mut arg_prepare = quote! {
            let mut extractor = #extractor::new(&mut args);
        };
        let mut call_args = TokenStream::new();

        for (i, ty) in self.args.iter().enumerate() {
            let mut argname = syn::Ident::new(&format!("arg{}", i), Span::call_site());

            if Some(i) == self.this {
                let from_value_out = &config.from_value_output;

                arg_prepare.extend(quote! {
                    let #argname = <#ty as #from_value_out>::from_value_out(this.copy(), realm)?;
                });
            } else if Some(i) == self.realm {
                argname = syn::Ident::new("realm", Span::call_site());
            } else {
                arg_prepare.extend(quote! {
                    let #argname = #extract_value::<#ty>::extract(&mut extractor, realm)?;
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
            if let Some((def, null)) = proto_default {
                let env = &config.env_path;

                let f = if *null {
                    quote! {null_proto_default()}
                } else {
                    quote! {proto_default(realm)}
                };

                quote! {
                    let mut guard = None;
                    let mut def = None::<Self>;

                    let this = if this.as_object() == Ok(realm.intrinsics.clone_public().#def.get(realm)?) {
                        &*def.insert(#env::utils::ProtoDefault::#f)
                    } else {
                        let this: yavashark_garbage::OwningGcGuard<_, Self> = FromValue::from_value(this)?;

                        &*guard.insert(this)
                    };
                }
            } else {
                quote! {
                    let this: yavashark_garbage::OwningGcGuard<_, Self> = FromValue::from_value(this)?;
                }
            }
        } else {
            TokenStream::new()
        };

        let name = self
            .js_name
            .clone()
            .map(|js_name| quote! {#js_name})
            .unwrap_or_else(|| quote! {stringify!(#name)});

        let mut length = self.args.len();

        if self.this.is_some() {
            length -= 1;
        }
        if self.realm.is_some() {
            length -= 1;
        }

        let optionals = self
            .args
            .iter()
            .filter(|arg| {
                if let syn::Type::Path(path) = deref_type(arg) {
                    path.path
                        .segments
                        .first()
                        .map(|seg| &seg.ident.to_string() == "Option")
                        .unwrap_or(false)
                } else {
                    false
                }
            })
            .count();

        length -= optionals;

        quote! {
            #native_function::with_proto_and_len(#name.as_ref(), |mut args, mut this, realm| {
                #arg_prepare
                #prepare_receiver
                #call.try_into_value(realm)
            }, realm.intrinsics.func.clone(), #length, realm)
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
