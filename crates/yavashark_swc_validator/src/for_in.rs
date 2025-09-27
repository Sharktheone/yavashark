use crate::Validator;
use swc_ecma_ast::{ForHead, ForInStmt, ObjectPatProp, Pat};

impl Validator {
    pub fn validate_for_in(for_in: &ForInStmt) -> Result<(), String> {
        match &for_in.left {
            ForHead::VarDecl(var_decl) => {
                for decl in &var_decl.decls {
                    if decl.init.is_some() {
                        return Err(
                            "ForInStmt variable declarations cannot have initializers"
                                .to_string(),
                        );
                    }

                    if pattern_has_initializer(&decl.name) {
                        return Err(
                            "ForInStmt binding patterns cannot contain initializers".to_string()
                        );
                    }

                    Self::validate_pat(&decl.name)?;
                }
            }
            ForHead::UsingDecl(using_decl) => {
                for decl in &using_decl.decls {
                    if decl.init.is_some() {
                        return Err(
                            "ForInStmt using declarations cannot have initializers".to_string(),
                        );
                    }

                    if pattern_has_initializer(&decl.name) {
                        return Err(
                            "ForInStmt binding patterns cannot contain initializers".to_string()
                        );
                    }

                    Self::validate_pat(&decl.name)?;
                }
            }
            ForHead::Pat(pat) => {
                if matches!(&**pat, Pat::Expr(expr) if expr.is_assign() ) {
                    return Err("ForInStmt left side cannot be an expression".to_string());
                }

                if pattern_has_initializer(pat) {
                    return Err(
                        "ForInStmt binding patterns cannot contain initializers".to_string()
                    );
                }

                Self::validate_pat(pat)?;
            }
        }

        Self::validate_expr(&for_in.right)?;

        Self::validate_statement(&for_in.body)
    }
}

fn pattern_has_initializer(pat: &Pat) -> bool {
    match pat {
        Pat::Assign(_) => true,
        Pat::Array(array) => array.elems.iter().flatten().any(pattern_has_initializer),
        Pat::Rest(rest) => pattern_has_initializer(&rest.arg),
        Pat::Object(object) => object.props.iter().any(|prop| match prop {
            ObjectPatProp::KeyValue(kv) => pattern_has_initializer(&kv.value),
            ObjectPatProp::Assign(assign) => assign.value.is_some(),
            ObjectPatProp::Rest(rest) => pattern_has_initializer(&rest.arg),
        }),
        _ => false,
    }
}