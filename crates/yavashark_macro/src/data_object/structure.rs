mod properties;

use crate::config::Config;
use proc_macro2::TokenStream;
use quote::quote;
use syn::ItemStruct;

pub fn data_struct(mut s: ItemStruct) -> syn::Result<TokenStream> {
    let properties = properties::parse_properties(&mut s.fields)?;

    let config = Config::new(s.struct_token.span);

    let from_value_output = &config.from_value_output;
    let try_into_value = &config.try_into_value;
    let res = &config.res;
    let value = &config.value;
    let realm = &config.realm;
    let object = &config.object;

    let struct_name = &s.ident;

    let mut gen_props = TokenStream::new();

    for prop in &properties {
        let prop_id = &prop.id;
        let prop_name = if let Some(name_expr) = &prop.name {
            quote! { #name_expr }
        } else {
            quote! { stringify!(#prop_id) }
        };

        let ty = &prop.ty;

        let get_val = if prop.required {
            let error = &config.error;
            quote! {
                obj.get_opt(#prop_name, realm)?
                .ok_or_else(|| #error::ty(concat!("Missing required property '", stringify!(#prop_name), "'")))?
            }
        } else {
            quote! {
                obj.get(#prop_name, realm)?
            }
        };

        gen_props.extend(quote! {
            let #prop_id = <#ty as #from_value_output>::from_value_out(#get_val, realm)?;
        });
    }

    let prop_names = properties.iter().map(|prop| &prop.id);

    let mut into_props = TokenStream::new();

    for prop in &properties {
        let prop_id = &prop.id;
        let prop_name = if let Some(name_expr) = &prop.name {
            quote! { #name_expr }
        } else {
            quote! { stringify!(#prop_id) }
        };

        into_props.extend(quote! {
            let value = #try_into_value::try_into_value(self.#prop_id, realm)?;
            obj.set(#prop_name, value, realm)?;
        });
    }

    Ok(quote! {
        #s

        impl #from_value_output for #struct_name {
            type Output = Self;

            fn from_value_out(value: #value, realm: &mut #realm) -> #res<Self::Output> {
                let obj = value.to_object()?;

                #gen_props

                Ok(Self {
                    #(
                        #prop_names
                    ),*
                })
            }
        }

        impl #try_into_value for #struct_name {
            fn try_into_value(self, realm: &mut #realm) -> #res<#value> {
                let obj = #object::new(realm);


                #into_props


                Ok(obj.into())
            }
        }
    })
}
