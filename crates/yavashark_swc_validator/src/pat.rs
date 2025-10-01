use crate::Validator;
use swc_ecma_ast::{ArrayPat, ObjectPat, ObjectPatProp, Pat};

impl<'a> Validator<'a> {
    pub fn validate_pat(&mut self, pat: &'a Pat) -> Result<(), String> {
        self.validate_pat_internal(pat, &mut None)
    }

    pub fn validate_pat_dup(&mut self, pat: &'a Pat, check_dups: bool) -> Result<(), String> {
        let mut idents = if check_dups { Some(Vec::new()) } else { None };

        self.validate_pat_internal(pat, &mut idents)
    }

    pub fn validate_pat_internal(
        &mut self,
        pat: &'a Pat,
        idents: &mut Option<Vec<&'a str>>,
    ) -> Result<(), String> {
        match pat {
            Pat::Ident(ident) => {
                if let Some(idents) = idents {
                    if ident.id.as_ref() == "_" {
                        return Ok(());
                    }

                    if idents.contains(&&*ident.id.sym) {
                        return Err(format!(
                            "Identifier '{}' has already been declared",
                            ident.id.sym
                        ));
                    }

                    idents.push(&ident.id.sym);
                }

                self.validate_ident(&ident.id)?;
            }
            Pat::Array(array) => self.validate_array_pat(array, idents)?,
            Pat::Rest(rest) => self.validate_pat_internal(&rest.arg, idents)?,
            Pat::Object(object) => self.validate_object_pat(object, idents)?,
            Pat::Assign(assign) => {
                self.validate_pat_internal(&assign.left, idents)?;
                self.validate_expr(&assign.right)?;
            }
            Pat::Expr(expr) => self.validate_expr(expr)?,
            Pat::Invalid(_) => return Err("Invalid pattern".to_string()),
        }

        Ok(())
    }

    pub fn validate_object_pat(
        &mut self,
        object: &'a ObjectPat,
        idents: &mut Option<Vec<&'a str>>,
    ) -> Result<(), String> {
        let mut assert_last = false;

        for prop in &object.props {
            if assert_last {
                return Err(
                    "Object rest element must be last element in object pattern".to_string()
                );
            }

            match prop {
                ObjectPatProp::KeyValue(kv) => {
                    self.validate_pat_internal(&kv.value, idents)?;
                }
                ObjectPatProp::Assign(assign) => {
                    self.validate_ident(&assign.key)?;
                }
                ObjectPatProp::Rest(rest) => {
                    assert_last = true;
                    self.validate_pat_internal(&rest.arg, idents)?;
                }
            }
        }

        Ok(())
    }

    pub fn validate_array_pat(
        &mut self,
        array: &'a ArrayPat,
        idents: &mut Option<Vec<&'a str>>,
    ) -> Result<(), String> {
        let mut assert_last = false;

        for elem in &array.elems {
            if assert_last {
                return Err("Elements after a rest pattern are not allowed".to_string());
            }

            if let Some(elem) = elem {
                if elem.is_rest() {
                    assert_last = true;
                }

                self.validate_pat_internal(elem, idents)?;
            }
        }

        Ok(())
    }
}

// pub fn pattern_has_initializer(pat: &Pat) -> bool {
//     match pat {
//         Pat::Assign(_) => true,
//         Pat::Array(array) => array.elems.iter().flatten().any(pattern_has_initializer),
//         Pat::Rest(rest) => pattern_has_initializer(&rest.arg),
//         Pat::Object(object) => object.props.iter().any(|prop| match prop {
//             ObjectPatProp::KeyValue(kv) => pattern_has_initializer(&kv.value),
//             ObjectPatProp::Assign(assign) => assign.value.is_some(),
//             ObjectPatProp::Rest(rest) => pattern_has_initializer(&rest.arg),
//         }),
//         _ => false,
//     }
// }
