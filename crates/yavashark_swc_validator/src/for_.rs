use crate::Validator;
use swc_ecma_ast::{ForStmt, VarDeclOrExpr};
use crate::utils::single_stmt_contains_decl;

impl Validator {
    pub fn validate_for(for_: &ForStmt) -> Result<(), String> {
        if let Some(init) = &for_.init {
            match init {
                VarDeclOrExpr::VarDecl(var_decl) => {
                    for decl in &var_decl.decls {
                        Self::validate_pat(&decl.name)?;

                        if let Some(init) = &decl.init {
                            Self::validate_expr(init)?;
                        }
                    }
                }
                VarDeclOrExpr::Expr(expr) => {
                    Self::validate_expr(expr)?;
                }
            }
        }

        if let Some(test) = &for_.test {
            Self::validate_expr(test)?;
        }

        if let Some(update) = &for_.update {
            Self::validate_expr(update)?;
        }


        if single_stmt_contains_decl(&for_.body) {
            return Err("Lexical declaration cannot appear in a single-statement context".to_string());
        }

        Self::validate_statement(&for_.body)
    }
}
