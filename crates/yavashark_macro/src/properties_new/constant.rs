use crate::properties_new::MaybeStatic;
use proc_macro2::TokenStream;
use quote::quote;
use syn::Expr;

#[derive(Clone)]
pub struct Constant {
    pub name: syn::Ident,
    pub js_name: Option<Expr>,
    pub writable: bool,
    pub enumerable: bool,
    pub configurable: bool,
}

impl Constant {
    /// Generate token stream for constant registration.
    pub fn init_tokens(
        &self,
        _config: &crate::config::Config,
        self_ty: TokenStream,
    ) -> TokenStream {
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
    let mut is_both = false;
    let mut writable = false;
    let mut enumerable = false;
    let mut configurable = false;

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
        } else if attr.path().is_ident("both") {
            is_both = true;
            is_static = false;
            return false;
        } else if attr.path().is_ident("writable") {
            writable = true;
            return false;
        } else if attr.path().is_ident("enumerable") {
            enumerable = true;
            return false;
        } else if attr.path().is_ident("configurable") {
            configurable = true;
            return false;
        }

        true
    });

    if let Some(e) = error {
        return Err(e);
    }

    Ok(if is_static {
        MaybeStatic::Static
    } else if is_both {
        MaybeStatic::Both
    } else {
        MaybeStatic::Impl
    }(Constant {
        name: constant.ident.clone(),
        js_name,
        writable,
        enumerable,
        configurable,
    }))
}
