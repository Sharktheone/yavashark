use crate::config::Config;
use crate::inline_props::property::{Kind, Property};
use quote::quote;

pub fn generate_values(props: &[Property], config: &Config) -> proc_macro2::TokenStream {
    let env = &config.env_path;
    let into_value = &config.into_value;
    let res = &config.res;
    let realm = &config.realm;

    let mut prop_items = Vec::with_capacity(props.len());

    let mut config_idx = 0usize;
    let mut write_idx = 0usize;

    let has_configurable = props.iter().any(|p| p.configurable);
    let has_writable = props.iter().any(|p| !p.readonly);

    for prop in props {
        if prop.kind == Kind::Setter {
            if prop.configurable {
                config_idx += 1;
            }
            if !prop.readonly {
                write_idx += 1;
            }
            continue;
        }

        let field = &prop.field;

        let c = config_idx;
        if prop.configurable {
            config_idx += 1;
        }

        let w = write_idx;
        if !prop.readonly {
            write_idx += 1;
        }

        let partial_get = if prop.partial {
            quote! {
                .get(realm)?
            }
        } else {
            quote! {}
        };

        let get = if prop.copy && !prop.readonly {
            quote! {
                #partial_get .get()
            }
        } else if !prop.copy && !prop.readonly {
            quote! {
                #partial_get .borrow().clone()
            }
        } else {
            quote! {
                #partial_get .clone()
            }
        };

        let attributes = prop.attributes(config);

        let value_expr = if prop.kind == Kind::Getter {
            quote! {
                #env::value::Property::Getter(self.#field #get, #attributes)
            }
        } else {
            quote! {
                {
                let val = #into_value::into_value(self.#field #get);
                #env::value::Property::Value(val, #attributes)
                }
            }
        };

        let value_expr = if !has_configurable && !has_writable {
            value_expr
        } else if prop.configurable && !prop.readonly {
            quote::quote! {
                if (self.__deleted_properties.get() & (1 << #c)) == 0 && (self.__written_properties.get() & (1 << #w)) == 0 {
                    Some(#value_expr)
                } else {
                    None
                }
            }
        } else if prop.configurable && prop.readonly {
            quote::quote! {
                if (self.__deleted_properties.get() & (1 << #c)) == 0 {
                    Some(#value_expr)
                } else {
                    None
                }
            }
        } else if !prop.readonly {
            quote::quote! {
                if (self.__written_properties.get() & (1 << #w)) == 0 {
                    Some(#value_expr)
                } else {
                    None
                }
            }
        } else {
            quote::quote! {
                Some(#value_expr)
            }
        };

        prop_items.push(value_expr);
    }

    if prop_items.is_empty() {
        return quote::quote! {
            #[inline(always)]
            fn values(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#env::value::Property)>> {
                ::core::result::Result::Ok(::core::iter::empty())
            }
        };
    }

    let configurable_flatten = if has_configurable {
        quote::quote! {
            .flatten()
        }
    } else {
        quote::quote! {}
    };

    quote::quote! {
        #[inline(always)]
        fn values(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=#env::value::Property>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
                #configurable_flatten
            )
        }
    }
}

pub fn generate_enumerable_values(props: &[Property], config: &Config) -> proc_macro2::TokenStream {
    let env = &config.env_path;
    let into_value = &config.into_value;
    let res = &config.res;
    let realm = &config.realm;

    let mut prop_items = Vec::with_capacity(props.len());

    let mut config_idx = 0usize;
    let mut write_idx = 0usize;

    let has_configurable = props.iter().any(|p| p.configurable);
    let has_writable = props.iter().any(|p| !p.readonly);

    for prop in props {
        if prop.kind != Kind::Getter && !prop.enumerable {
            if prop.configurable {
                config_idx += 1;
            }
            continue;
        }

        let field = &prop.field;

        let c = config_idx;
        if prop.configurable {
            config_idx += 1;
        }

        let w = write_idx;
        if !prop.readonly {
            write_idx += 1;
        }

        let partial_get = if prop.partial {
            quote! {
                .get(realm)?
            }
        } else {
            quote! {}
        };

        let get = if prop.copy && !prop.readonly {
            quote! {
                #partial_get .get()
            }
        } else if !prop.copy && !prop.readonly {
            quote! {
                #partial_get .borrow().clone()
            }
        } else {
            quote! {
                #partial_get .clone()
            }
        };

        let attributes = prop.attributes(config);

        let value_expr = if prop.kind == Kind::Getter {
            quote! {
                #env::value::Property::Getter(self.#field #get, #attributes)
            }
        } else {
            quote! {
                {
                let val = #into_value::into_value(self.#field #get);
                #env::value::Property::Value(val, #attributes)
                }
            }
        };

        let value_expr = if !has_configurable && !has_writable {
            value_expr
        } else if prop.configurable && !prop.readonly {
            quote::quote! {
                if (self.__deleted_properties.get() & (1 << #c)) == 0 && (self.__written_properties.get() & (1 << #w)) == 0 {
                    Some(#value_expr)
                } else {
                    None
                }
            }
        } else if prop.configurable && prop.readonly {
            quote::quote! {
                if (self.__deleted_properties.get() & (1 << #c)) == 0 {
                    Some(#value_expr)
                } else {
                    None
                }
            }
        } else if !prop.readonly {
            quote::quote! {
                if (self.__written_properties.get() & (1 << #w)) == 0 {
                    Some(#value_expr)
                } else {
                    None
                }
            }
        } else {
            quote::quote! {
                Some(#value_expr)
            }
        };

        prop_items.push(value_expr);
    }

    if prop_items.is_empty() {
        return quote::quote! {
            #[inline(always)]
            fn enumerable_values(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#env::value::Property)>> {
                ::core::result::Result::Ok(::core::iter::empty())
            }
        };
    }

    let configurable_flatten = if has_configurable {
        quote::quote! {
            .flatten()
        }
    } else {
        quote::quote! {}
    };

    quote::quote! {
        #[inline(always)]
        fn enumerable_values(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=#env::value::Property>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
                #configurable_flatten
            )
        }
    }
}
