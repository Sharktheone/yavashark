use crate::config::Config;
use crate::inline_props::property::{Kind, Name, Property};

pub fn generate_keys(props: &[Property], config: &Config) -> proc_macro2::TokenStream {
    let res = &config.res;
    let property_key = &config.property_key;
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

            continue;
        }

        let key = &prop.name;

        let c = config_idx;
        if prop.configurable {
            config_idx += 1;
        }

        let w = write_idx;
        if !prop.readonly {
            write_idx += 1;
        }

        let value_expr = match key {
            Name::Str(s) => {
                quote::quote! {
                    #property_key::from_static(#s)
                }
            }
            Name::Symbol(sym) => {
                quote::quote! {
                    #property_key::from_symbol(#sym.clone())
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
            fn keys(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#property_key)>> {
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
        fn keys(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#property_key)>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
                #configurable_flatten
            )
        }
    }
}

pub fn generate_enumerable_keys(props: &[Property], config: &Config) -> proc_macro2::TokenStream {
    let res = &config.res;
    let property_key = &config.property_key;
    let realm = &config.realm;

    let mut prop_items = Vec::with_capacity(props.len());

    let has_configurable = props.iter().any(|p| p.configurable);
    let has_writable = props.iter().any(|p| !p.readonly);

    let mut config_idx = 0usize;
    let mut write_idx = 0usize;

    for prop in props {
        if prop.kind == Kind::Setter || !prop.enumerable {
            if prop.configurable {
                config_idx += 1;
            }

            if !prop.readonly {
                write_idx += 1;
            }

            continue;
        }

        let key = &prop.name;

        let c = config_idx;
        if prop.configurable {
            config_idx += 1;
        }

        let w = write_idx;
        if !prop.readonly {
            write_idx += 1;
        }

        let value_expr = match key {
            Name::Str(s) => {
                quote::quote! {
                    #property_key::from_static(#s)
                }
            }
            Name::Symbol(sym) => {
                quote::quote! {
                    #property_key::from_symbol(#sym.clone())
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
            fn enumerable_keys(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#property_key)>> {
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
        fn enumerable_keys(&self, realm: &mut #realm) -> #res<impl ::core::iter::Iterator<Item=(#property_key)>> {
            ::core::result::Result::Ok(
                ::core::iter::IntoIterator::into_iter([
                    #(#prop_items),*
                ])
                #configurable_flatten
            )
        }
    }
}
