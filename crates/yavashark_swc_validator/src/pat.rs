use crate::Validator;
use swc_ecma_ast::{ObjectPatProp, Pat};

impl Validator {
    pub fn validate_pat(pat: &Pat) -> Result<(), String> {
        match pat {
            Pat::Ident(ident) => Self::validate_ident(&ident.id)?,
            Pat::Array(array) => {
                let mut assert_last = false;

                for elem in &array.elems {
                    if assert_last {
                        return Err("Elements after a rest pattern are not allowed".to_string());
                    }

                    if let Some(elem) = elem {
                        if elem.is_rest() {
                            assert_last = true;
                        }

                        Self::validate_pat(elem)?;
                    }
                }
            }
            Pat::Rest(rest) => Self::validate_pat(&rest.arg)?,
            Pat::Object(object) => {
                let mut assert_last = false;
                
                for prop in &object.props {
                    if assert_last {
                        return Err("Object rest element must be last element in object pattern".to_string());
                    }
                    
                    match prop {
                        ObjectPatProp::KeyValue(kv) => {
                            Self::validate_pat(&kv.value)?;
                        }
                        ObjectPatProp::Assign(assign) => {
                            Self::validate_ident(&assign.key)?;
                        }
                        ObjectPatProp::Rest(rest) => {
                            assert_last = true;
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
