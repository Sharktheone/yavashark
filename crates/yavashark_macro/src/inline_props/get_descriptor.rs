use crate::config::Config;
use crate::inline_props::property::{Kind, Name, Property};

pub fn generate_get_descriptor(props: &[Property], config: &Config) -> proc_macro2::TokenStream {
    let internal_property_key = &config.internal_property_key;
    let realm = &config.realm;
    let env = &config.env_path;
    let into_value = &config.into_value;
    let res = &config.res;

    let mut string_arms = Vec::with_capacity(props.len());
    let mut symbols = Vec::new();

    let has_configurable = props.iter().any(|prop| prop.configurable);

    let mut config_idx = 0usize;
    let mut write_idx = 0usize;

    for prop in props {
        let key = &prop.name;
        let field = &prop.field;

        let c = config_idx;
        if prop.configurable {
            config_idx += 1;
        }

        let w = write_idx;
        if !prop.readonly {
            write_idx += 1;
        }

        let enumerable = prop.enumerable;
        let configurable = prop.configurable;
        let writable = !prop.readonly && matches!(prop.kind, Kind::Property);

        let partial_get = if prop.partial {
            quote::quote! { .get(realm)? }
        } else {
            quote::quote! {}
        };

        let accessor_value = if prop.copy {
            quote::quote! { self.#field #partial_get }
        } else {
            quote::quote! { self.#field #partial_get .clone() }
        };

        let descriptor_expr = match prop.kind {
            Kind::Getter => {
                quote::quote! {
                    let getter = #accessor_value;
                    return ::core::result::Result::Ok(::core::option::Option::Some(#env::value::PropertyDescriptor::Accessor {
                        get: ::core::option::Option::Some(getter),
                        set: ::core::option::Option::None,
                        enumerable: #enumerable,
                        configurable: #configurable,
                    }));
                }
            }
            Kind::Setter => {
                quote::quote! {
                    let setter = #accessor_value;
                    return ::core::result::Result::Ok(::core::option::Option::Some(#env::value::PropertyDescriptor::Accessor {
                        get: ::core::option::Option::None,
                        set: ::core::option::Option::Some(setter),
                        enumerable: #enumerable,
                        configurable: #configurable,
                    }));
                }
            }
            Kind::Property => {
                if prop.readonly {
                    quote::quote! {
                        let value = #into_value::into_value(self.#field #partial_get.clone());
                        return ::core::result::Result::Ok(::core::option::Option::Some(#env::value::PropertyDescriptor::Data {
                            value,
                            writable: #writable,
                            enumerable: #enumerable,
                            configurable: #configurable,
                        }));
                    }
                } else if prop.copy {
                    quote::quote! {
                        if (self.__written_properties.get() & (1 << #w)) != 0 {
                            return ::core::result::Result::Ok(::core::option::Option::None);
                        }

                        let value = #into_value::into_value(self.#field #partial_get .get());
                        return ::core::result::Result::Ok(::core::option::Option::Some(#env::value::PropertyDescriptor::Data {
                            value,
                            writable: #writable,
                            enumerable: #enumerable,
                            configurable: #configurable,
                        }));
                    }
                } else {
                    quote::quote! {
                        if (self.__written_properties.get() & (1 << #w)) != 0 {
                            return ::core::result::Result::Ok(::core::option::Option::None);
                        }

                        let value = self.#field #partial_get .borrow().clone();
                        let value = #into_value::into_value(value);
                        return ::core::result::Result::Ok(::core::option::Option::Some(#env::value::PropertyDescriptor::Data {
                            value,
                            writable: #writable,
                            enumerable: #enumerable,
                            configurable: #configurable,
                        }));
                    }
                }
            }
        };

        let descriptor_expr = if has_configurable && prop.configurable {
            quote::quote! {
                if (self.__deleted_properties.get() & (1 << #c)) == 0 {
                    #descriptor_expr
                } else {
                    return ::core::result::Result::Ok(::core::option::Option::None);
                }
            }
        } else {
            descriptor_expr
        };

        match key {
            Name::Str(s) => {
                string_arms.push(quote::quote! {
                    #s => {
                        #descriptor_expr
                    }
                });
            }
            Name::Symbol(sym) => {
                symbols.push(quote::quote! {
                    if symbol == #sym {
                        #descriptor_expr
                    }
                });
            }
        }
    }

    let str_check = if !string_arms.is_empty() {
        quote::quote! {
            #internal_property_key::String(str) => {
                match str.as_str() {
                    #(#string_arms)*
                    _ => {}
                }
            },
        }
    } else {
        quote::quote! {}
    };

    let symbol_check = if !symbols.is_empty() {
        quote::quote! {
            #internal_property_key::Symbol(symbol) => {
                #(#symbols)*
            },
        }
    } else {
        quote::quote! {}
    };

    quote::quote! {
        #[inline(always)]
        fn get_descriptor(&self, key: &#internal_property_key, realm: &mut #realm) -> #res<::core::option::Option<#env::value::PropertyDescriptor>> {
            match key {
                #str_check
                #symbol_check
                _ => {}
            }

            ::core::result::Result::Ok(::core::option::Option::None)
        }
    }
}
