use crate::properties_new::MaybeStatic;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

pub struct Constant {
    pub name: syn::Ident,
    pub js_name: Option<Expr>,
}

impl Constant {
    /// Generate token stream for constant registration.
    pub fn init_tokens(&self, _config: &crate::config::Config, self_ty: TokenStream) -> TokenStream {
        let name = &self.name;

        quote! {
            #self_ty::#name.into_value()
        }
    }
}

pub fn parse_constant(
    constant: &mut syn::ImplItemConst,
) -> Result<MaybeStatic<Constant>, syn::Error> {
    let mut js_name = None;

    let mut is_static = true;

    let mut error = None;
    constant.attrs.retain_mut(|attr| {
        if attr.path().is_ident("prop") {
            let args = match attr.parse_args() {
                Ok(args) => args,
                Err(e) => {
                    error = Some(e);
                    return false;
                }
            };

            js_name = Some(args);
            return false;
        } else if attr.path().is_ident("nonstatic") {
            is_static = false;
            return false;
        }

        true
    });

    if let Some(e) = error {
        return Err(e);
    }

    Ok(if is_static {
        MaybeStatic::Static
    } else {
        MaybeStatic::Impl
    }(Constant {
        name: constant.ident.clone(),
        js_name,
    }))
}
