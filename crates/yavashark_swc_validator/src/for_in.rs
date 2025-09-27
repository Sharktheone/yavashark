use crate::Validator;
use swc_ecma_ast::{ForHead, ForInStmt, Pat, VarDeclKind};
use crate::utils::single_stmt_contains_decl;

impl Validator {
    pub fn validate_for_in(for_in: &ForInStmt) -> Result<(), String> {
        match &for_in.left {
            ForHead::VarDecl(var_decl) => {
                for decl in &var_decl.decls {
                    if decl.init.is_some() {
                        return Err(
                            "ForInStmt variable declarations cannot have initializers".to_string()
                        );
                    }

                    Self::validate_pat_dup(&decl.name, var_decl.kind != VarDeclKind::Var)?;
                }
            }
            ForHead::UsingDecl(using_decl) => {
                for decl in &using_decl.decls {
                    if decl.init.is_some() {
                        return Err(
                            "ForInStmt using declarations cannot have initializers".to_string()
                        );
                    }

                    Self::validate_pat(&decl.name)?;
                }
            }
            ForHead::Pat(pat) => {
                if matches!(&**pat, Pat::Expr(expr) if expr.is_assign() ) {
                    return Err("ForInStmt left side cannot be an expression".to_string());
                }

                Self::validate_pat(pat)?;
            }
        }

        Self::validate_expr(&for_in.right)?;

        if single_stmt_contains_decl(&for_in.body) {
            return Err("Lexical declaration cannot appear in a single-statement context".to_string());
        }

        Self::validate_statement(&for_in.body)
    }
}

