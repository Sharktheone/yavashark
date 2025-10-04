mod property;
mod set_property;
mod get_property;
mod contains_property;
mod properties;
mod keys;
mod values;

use proc_macro2::TokenStream;
use crate::config::Config;
use crate::inline_props::property::{Kind, Property};
use syn::spanned::Spanned;

pub fn inline_props(
    attrs: proc_macro::TokenStream,
    item: proc_macro::TokenStream,
) -> proc_macro::TokenStream {
    let mut input: syn::ItemStruct = syn::parse_macro_input!(item);

    let config = Config::new(input.span());

    let fields = if let syn::Fields::Named(fields) = &mut input.fields {
        &mut fields.named
    } else {
        return syn::Error::new_spanned(input, "Expected a struct with named fields")
            .to_compile_error()
            .into();
    };

    let mut props = Vec::with_capacity(fields.len());

    for field in fields.iter_mut() {
        let mut prop = match Property::from_field(field) {
            Ok(f) => f,
            Err(e) => return e.to_compile_error().into(),
        };

        let ty = &field.ty;

        if matches!(prop.kind, Kind::Getter | Kind::Setter) {
            prop.readonly = true;
        }

        if !prop.readonly {
            if prop.copy {
                if prop.partial {
                    field.ty = update_partial_type(field.ty.clone(), quote::quote! { ::core::cell::Cell });
                } else {
                    field.ty = syn::parse_quote! {
                    ::core::cell::Cell<#ty>
                };
                }
            } else if prop.partial {
                field.ty = update_partial_type(field.ty.clone(), quote::quote! { ::core::cell::RefCell });
            } else {
                //TODO: we would want to only have one cell for all mutable props -> we need to add a Mutable<#StructName> struct
                field.ty = syn::parse_quote! {
                ::core::cell::RefCell<#ty>
            };
            }
        }

        props.push(prop);
    }

    let prop_impl = generate_impl(&input.ident, &props, &config);


    quote::quote! {
        #input

        #prop_impl
    }.into()
}


fn generate_impl(
    struct_name: &syn::Ident,
    props: &[Property],
    config: &Config,
) -> TokenStream {
    let set_property = set_property::generate_set_property(props, config);
    let get_property = get_property::generate_get_property(props, config);
    let contains_property = contains_property::generate_contains_property(props, config);
    let properties = properties::generate_properties(props, config);
    let keys = keys::generate_keys(props, config);
    let values = values::generate_values(props, config);
    let enumerable_values = values::generate_enumerable_values(props, config);
    let enumerable_properties = properties::generate_enumerable_properties(props, config);

    let env = &config.env_path;

    quote::quote! {
        impl #env::inline_props::PropertiesHook for #struct_name {
            #set_property
            #get_property
            #contains_property
            #properties
            #keys
            #values
            #enumerable_values
            #enumerable_properties


            fn gc_refs(&self) -> impl Iterator<Item = yavashark_garbage::GcRef<#env::value::BoxedObj>> {
                ::core::iter::empty()
            }
        }
    }
}

fn update_partial_type(mut ty: syn::Type, wrapper: TokenStream) -> syn::Type {
    if let syn::Type::Path(type_path) = &mut ty {
        if let Some(segment) = type_path.path.segments.last_mut() {
            if segment.ident == "Partial" {
                if let syn::PathArguments::AngleBracketed(args) = &mut segment.arguments {
                    if let Some(syn::GenericArgument::Type(inner_ty)) = args.args.first_mut() {
                        *inner_ty = syn::parse_quote! {
                            #wrapper<#inner_ty>
                        };

                    }
                }
            }
        }
    }

    ty
}