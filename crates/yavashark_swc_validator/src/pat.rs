use swc_ecma_ast::{ObjectPatProp, Pat};
use crate::Validator;

impl Validator {
    pub fn validate_pat(pat: &Pat) -> Result<(), String> {
        match pat {
            Pat::Ident(ident) => Self::validate_ident(&ident.id)?,
            Pat::Array(array) => {
                for elem in array.elems.iter().flatten() {
                    Self::validate_pat(elem)?;
                }
            }
            Pat::Rest(rest) => Self::validate_pat(&rest.arg)?,
            Pat::Object(object) => {
                for prop in &object.props {
                    match prop {
                        ObjectPatProp::KeyValue(kv) => {
                            Self::validate_pat(&kv.value)?;
                        }
                        ObjectPatProp::Assign(assign) => {
                            Self::validate_ident(&assign.key)?;
                        }
                        ObjectPatProp::Rest(rest) => {
                            Self::validate_pat(&rest.arg)?;
                        }
                    }
                }
            }
            Pat::Assign(assign) => {
                Self::validate_pat(&assign.left)?;
                Self::validate_expr(&assign.right)?;
            }
            Pat::Expr(expr) => Self::validate_expr(expr)?,
            Pat::Invalid(_) => return Err("Invalid pattern".to_string()),
        }

        Ok(())
    }
}
