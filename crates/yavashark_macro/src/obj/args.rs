use darling::ast::NestedMeta;
use darling::FromMeta;
use proc_macro2::Ident;
use quote::ToTokens;
use syn::spanned::Spanned;
use syn::{Error, FieldsNamed, Path};

#[derive(Debug, FromMeta)]
pub struct ObjArgs {
    #[darling(default)]
    pub function: bool,
    #[darling(default)]
    pub to_string: bool,
    #[darling(default)]
    pub name: bool,
    #[darling(default)]
    pub extends: Option<Path>,
    #[darling(default)]
    pub constructor: bool,
    #[darling(default)]
    pub direct: Direct,
}

#[derive(Debug, Default)]
pub struct Direct {
    pub fields: Vec<DirectItem>,
}

impl FromMeta for Direct {
    fn from_list(items: &[NestedMeta]) -> darling::Result<Self> {
        let mut fields = Vec::new();

        for item in items {
            let item = match item {
                NestedMeta::Meta(meta) => meta,
                NestedMeta::Lit(lit) => {
                    return Err(darling::Error::from(Error::new(
                        lit.span(),
                        "Unexpected literal",
                    )));
                }
            };

            let item = DirectItem::from_meta(item)?;

            fields.push(item);
        }

        Ok(Direct { fields })
    }
}

#[derive(Debug)]
pub struct DirectItem {
    pub field: Ident,
    pub rename: Option<Ident>,
}

impl FromMeta for DirectItem {
    fn from_meta(meta: &syn::Meta) -> darling::Result<Self> {
        let (field, rename) = match meta {
            syn::Meta::Path(path) => {
                let field = path
                    .get_ident()
                    .ok_or_else(|| darling::Error::custom("Expected ident"))?;

                (field.clone(), None)
            }
            syn::Meta::List(list) => {
                let field = list
                    .path
                    .get_ident()
                    .ok_or_else(|| darling::Error::custom("Expected ident"))?;

                let ident = syn::parse(list.tokens.clone().into())?;

                (field.clone(), Some(ident))
            }
            syn::Meta::NameValue(name_value) => {
                let field = name_value
                    .path
                    .get_ident()
                    .ok_or_else(|| darling::Error::custom("Expected ident"))?;

                let ident = syn::parse(name_value.value.to_token_stream().into())?;

                (field.clone(), Some(ident))
            }
        };

        Ok(DirectItem { field, rename })
    }
}

pub struct ItemArgs {
    pub gc: Vec<GcItem>,
    pub mutable_region: Vec<Ident>,
    pub primitive: Option<Ident>,
}

impl ItemArgs {
    pub fn from(fields: &mut FieldsNamed) -> syn::Result<Self> {
        let mut gc = Vec::new();
        let mut mutable_region = Vec::new();
        let mut primitive = None;

        for f in &mut fields.named {
            let mut err = None;
            f.attrs.retain_mut(|attr| {
                if attr.meta.path().is_ident("gc") {
                    let mut ty = true;
                    let mut func = None;
                    let mut multi = false;

                    if !matches!(attr.meta, syn::Meta::Path(_)) {
                        if let Err(e) =
                            attr.parse_nested_meta(|meta| {
                                if meta.path.is_ident("untyped") {
                                    ty = false;
                                    return Ok(());
                                }

                                if meta.path.is_ident("func") {
                                    func = Some(meta.path.get_ident().cloned().ok_or(
                                        syn::Error::new(meta.path.span(), "Expected ident"),
                                    )?);
                                    return Ok(());
                                }

                                if meta.path.is_ident("multi") {
                                    multi = true;
                                    return Ok(());
                                }

                                Err(syn::Error::new(meta.path.span(), "Unknown attribute"))
                            })
                        {
                            err = Some(e);
                            return false;
                        };
                    }

                    let id = match f
                        .ident
                        .as_ref()
                        .ok_or(syn::Error::new(attr.meta.span(), "Expected ident"))
                    {
                        Ok(id) => id,
                        Err(e) => {
                            err = Some(e);
                            return false;
                        }
                    }
                    .clone();

                    gc.push(GcItem {
                        name: id,
                        ty,
                        multi,
                        func,
                    });

                    return false;
                }

                if attr.meta.path().is_ident("mutable") {
                    let Ok(ident) = f
                        .ident
                        .clone()
                        .ok_or(syn::Error::new(attr.span(), "Expected ident"))
                    else {
                        err = Some(syn::Error::new(attr.span(), "Expected ident"));
                        return false;
                    };

                    mutable_region.push(ident);
                    return false;
                }

                if attr.meta.path().is_ident("primitive") {
                    let Ok(ident) = f
                        .ident
                        .clone()
                        .ok_or(syn::Error::new(attr.span(), "Expected ident"))
                    else {
                        err = Some(syn::Error::new(attr.span(), "Expected ident"));
                        return false;
                    };

                    primitive = Some(ident); //TODO: edge case, what when we have a field that is a primitive but not mutable and a field with the same name that is mutable?

                    return false;
                }
                true
            });

            if let Some(e) = err {
                return Err(e);
            }
        }

        Ok(Self {
            gc,
            mutable_region,
            primitive,
        })
    }
}

pub struct GcItem {
    pub name: Ident,
    pub ty: bool,
    pub multi: bool,
    pub func: Option<Ident>,
}
