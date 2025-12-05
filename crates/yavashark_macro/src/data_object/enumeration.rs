use crate::config::Config;
use crate::data_object::args::DataObjectArgs;
use proc_macro2::{Ident, Span, TokenStream};
use syn::spanned::Spanned;

pub fn data_enum(mut e: syn::ItemEnum, args: DataObjectArgs) -> syn::Result<TokenStream> {
    let mut variants = Vec::with_capacity(e.variants.len());

    for variant in e.variants.iter_mut() {
        let mut variant_name = variant.ident.to_string();

        if !variant_name.is_empty() {
            variant_name[0..1].make_ascii_lowercase();
        }

        variant.attrs.retain(|attr| {
            if attr.meta.path().is_ident("name") {
                let Ok(x) = attr.parse_args::<syn::LitStr>() else {
                    return true;
                };

                variant_name = x.value();
                return false;
            }

            true
        });

        variants.push((variant_name, variant.ident.clone()));
    }

    let enum_name = &e.ident;

    let config = Config::new(e.span());

    let error = &config.error;
    let into_value = &config.into_value;
    let from_value_output = &config.from_value_output;
    let value = &config.value;
    let res = &config.res;
    let realm = &config.realm;

    let from_cases = variants.iter().map(|(variant_name, variant_ident)| {
        quote::quote! {
            #variant_name => Ok(#enum_name::#variant_ident),
        }
    });

    let to_cases = variants.iter().map(|(variant_name, variant_ident)| {
        quote::quote! {
            #enum_name::#variant_ident => #variant_name,
        }
    });

    let err = match args.error.as_deref().unwrap_or("type") {
        "type" => Ident::new("ty", Span::call_site()),
        "range" => Ident::new("range", Span::call_site()),
        "syntax" => Ident::new("syntax", Span::call_site()),
        "uri" => Ident::new("uri", Span::call_site()),
        "reference" => Ident::new("reference", Span::call_site()),
        other => {
            return Err(syn::Error::new_spanned(
                e,
                format!("Invalid error type: {}", other),
            ));
        }
    };

    Ok(quote::quote! {
        #e

        impl std::str::FromStr for #enum_name {

            type Err = #error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_cases)*
                    _ => Err(#error::#err("Invalid enum variant string")),
                }
            }

        }

        impl #enum_name {
            pub fn as_str(&self) -> &'static str {
                match self {
                    #(#to_cases)*
                }
            }
        }

        impl #into_value for #enum_name {
            fn into_value(self) -> #value {
               #value::String(self.as_str().into())
            }
        }

        impl #from_value_output for #enum_name {
            type Output = Self;

            fn from_value_out(value: #value, realm: &mut #realm) -> #res<Self::Output> {
                let s = value.to_string(realm)?;

                <Self as std::str::FromStr>::from_str(&s)
            }
        }
    })
}
