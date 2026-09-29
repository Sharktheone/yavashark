use super::Checker;
use crate::{Result, ValidationError};
use swc_common::Spanned;
use swc_ecma_ast::{ArrayPat, ObjectPat, ObjectPatProp, Pat};

impl<'a> Checker<'a, '_> {
    pub(super) fn validate_pat(&mut self, pat: &'a Pat, kind: u8) -> Result<'a> {
        self.depth += 1;
        let result = if self.depth.is_multiple_of(16) {
            stacker::maybe_grow(256 * 1024, 2 * 1024 * 1024, || {
                self.validate_pat_inner(pat, kind)
            })
        } else {
            self.validate_pat_inner(pat, kind)
        };
        self.depth -= 1;
        result
    }

    pub(super) fn validate_pat_inner(&mut self, pat: &'a Pat, kind: u8) -> Result<'a> {
        match pat {
            Pat::Ident(i) => {
                if kind == 0 {
                    self.validate_binding(&i.id)?;
                } else {
                    self.declare_binding(&i.id, kind)?;
                }
            }
            Pat::Array(a) => {
                for (index, pat) in a.elems.iter().enumerate() {
                    if let Some(pat) = pat {
                        if matches!(pat, Pat::Rest(_)) && index + 1 != a.elems.len() {
                            return Err(ValidationError::new(
                                "Rest element must be last",
                                pat.span(),
                            ));
                        }

                        self.validate_pat(pat, kind)?;
                    }
                }
            }
            Pat::Object(o) => {
                for (index, pat) in o.props.iter().enumerate() {
                    match pat {
                        ObjectPatProp::KeyValue(k) => {
                            self.validate_key(&k.key)?;
                            self.validate_pat(&k.value, kind)?;
                        }
                        ObjectPatProp::Assign(a) => {
                            if kind == 0 {
                                self.validate_binding(&a.key.id)?;
                            } else {
                                self.declare_binding(&a.key.id, kind)?;
                            }

                            if let Some(e) = &a.value {
                                self.validate_expr(e)?;
                            }
                        }
                        ObjectPatProp::Rest(r) => {
                            if index + 1 != o.props.len()
                                || (kind != 0 && !matches!(&*r.arg, Pat::Ident(_)))
                            {
                                return Err(ValidationError::new(
                                    "Invalid object rest binding",
                                    r.span,
                                ));
                            }

                            self.validate_pat(&r.arg, kind)?;
                        }
                    }
                }
            }
            Pat::Rest(r) => {
                if matches!(&*r.arg, Pat::Assign(_)) {
                    return Err(ValidationError::new(
                        "Rest binding cannot have a default",
                        r.span,
                    ));
                }

                self.validate_pat(&r.arg, kind)?;
            }
            Pat::Assign(a) => {
                self.validate_pat(&a.left, kind)?;
                self.validate_expr(&a.right)?;
            }
            Pat::Expr(e) => {
                if kind != 0 {
                    return Err(ValidationError::new(
                        "Expression in binding pattern",
                        e.span(),
                    ));
                }

                self.validate_assignment_target(e)?;
            }
            Pat::Invalid(i) => return Err(ValidationError::new("Invalid binding pattern", i.span)),
        }

        Ok(())
    }

    pub(super) fn validate_array_assignment(&mut self, array: &'a ArrayPat) -> Result<'a> {
        for (i, e) in array.elems.iter().enumerate() {
            if let Some(e) = e {
                if matches!(e, Pat::Rest(_)) && i + 1 != array.elems.len() {
                    return Err(ValidationError::new("Rest must be last", e.span()));
                }

                self.validate_pat(e, 0)?;
            }
        }

        Ok(())
    }

    pub(super) fn validate_object_assignment(&mut self, object: &'a ObjectPat) -> Result<'a> {
        let len = object.props.len();

        for (i, object) in object.props.iter().enumerate() {
            match object {
                ObjectPatProp::KeyValue(k) => {
                    self.validate_key(&k.key)?;
                    self.validate_pat(&k.value, 0)?;
                }
                ObjectPatProp::Assign(a) => {
                    self.validate_binding(&a.key.id)?;

                    if let Some(e) = &a.value {
                        self.validate_expr(e)?;
                    }
                }
                ObjectPatProp::Rest(r) => {
                    if i + 1 != len {
                        return Err(ValidationError::new("Rest must be last", r.span));
                    }

                    self.validate_pat(&r.arg, 0)?;
                }
            }
        }

        Ok(())
    }
}
