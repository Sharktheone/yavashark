use proc_macro2::Ident;
use syn::{Attribute, Expr, Field, Path, Type};
use syn::spanned::Spanned;

pub struct Property {
    pub copy: bool,
    pub readonly: bool,
    pub configurable: bool,
    pub enumerable: bool,
    pub name: Name,
    pub field: Ident,
    pub kind: Kind,
    pub ty: Type,
}

#[derive(PartialEq, Eq, Copy, Clone, Debug)]
pub enum Kind {
    Property,
    Getter,
    Setter,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Name {
    Str(String),
    Symbol(Path),
}

impl Default for Property {
    fn default() -> Self {
        Self {
            copy: false,
            readonly: false,
            configurable: true,
            enumerable: true,
            name: Name::Str(String::new()),
            field: Ident::new("unknown", proc_macro2::Span::call_site()),
            kind: Kind::Property,
            ty: syn::parse_quote! { () }
        }
    }
}

impl Property {
    pub fn from_field(field: &mut Field) -> syn::Result<Self> {
        let mut flags = Property::default();

        let name = field.ident.as_ref().ok_or_else(|| {
            syn::Error::new(field.span(), "Expected named field")
        })?;

        flags.name = Name::Str(name.to_string());
        flags.field = name.clone();
        flags.ty = field.ty.clone();

        let mut copy_auto = true;

        field.attrs.retain(|attr| {
            let Some(id) = attr.meta.path().get_ident() else {
                return true
            };


            match id.to_string().as_str() {
                "copy" => {
                    flags.copy = true;
                    copy_auto = false;
                    false
                }
                "no_copy" => {
                    flags.copy = false;
                    copy_auto = false;
                    false
                }
                "readonly" => {
                    flags.readonly = true;
                    false
                }
                "no_configurable" => {
                    flags.configurable = false;
                    false
                }
                "no_enumerable" => {
                    flags.enumerable = false;
                    false
                }
                "prop" => {
                    let n = match attr.parse_args().map_err(|e| syn::Error::new(e.span(), e)) {
                        Ok(n) => n,
                        Err(_) => {
                            return true;
                        }
                    };


                    let name = match n {
                        Expr::Lit(expr_lit) => {
                            if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                                let value = lit_str.value();
                                if value.is_empty() {
                                    return true;
                                }

                                Name::Str(value)
                            } else {
                                return true;
                            }
                        }
                        Expr::Path(expr_path) => {
                            if expr_path.path.segments.is_empty() {
                                return true;
                            }

                            Name::Symbol(expr_path.path.clone())
                        }
                        _ => {
                            return true;
                        }
                    };

                    flags.name = name;

                    false
                },
                "get" => {
                    flags.kind = Kind::Getter;
                    false
                }
                "set" => {
                    flags.kind = Kind::Setter;
                    false
                }
                _ => true,
            }
        });

        if copy_auto {
            flags.copy = type_is_copy(&field.ty);
        }



        Ok(flags)
    }
}



fn type_is_copy(ty: &syn::Type) -> bool {
    match ty {
        syn::Type::Path(type_path) => {
            if type_path.qself.is_some() {
                return false;
            }
            if let Some(ident) = type_path.path.get_ident() {
                matches!(
                    ident.to_string().as_str(),
                    "u8" | "u16" | "u32" | "u64" | "u128" |
                    "i8" | "i16" | "i32" | "i64" | "i128" |
                    "f32" | "f64" | "bool" | "char"
                )
            } else {
                false
            }
        }
        _ => false,
    }
}
