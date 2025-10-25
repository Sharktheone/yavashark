use crate::config::Config;
use proc_macro2::TokenStream;
use syn::spanned::Spanned;

pub fn data_enum(mut e: syn::ItemEnum) -> syn::Result<TokenStream> {
    let mut variants = Vec::with_capacity(e.variants.len());

    for variant in e.variants.iter_mut() {
        let mut variant_name = variant.ident.to_string();

        if !variant_name.is_empty() {
            variant_name[0..1].make_ascii_lowercase();
        }

        variant.attrs.retain(|attr| {
            if attr.meta.path().is_ident("name") {
                let Ok(x) = attr.parse_args::<syn::LitStr>() else  {
                    return true
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
    let try_into_value = &config.try_into_value;
    let from_value_output = &config.from_value_output;
    let value = &config.value;
    let res = &config.res;
    let value_result = &config.value_result;
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

    Ok(quote::quote! {
        #e

        impl std::str::FromStr for #enum_name {

            type Err = #error;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                match s {
                    #(#from_cases)*
                    _ => Err(#error::ty("Invalid enum variant string")),
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

        impl #try_into_value for #enum_name {
            fn try_into_value(self, realm: &mut #realm) -> #value_result {
                Ok(#value::String(self.as_str().into()))
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
