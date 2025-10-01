use crate::Validator;
use crate::utils::single_stmt_contains_decl;
use swc_ecma_ast::{ForStmt, VarDeclKind, VarDeclOrExpr};

impl<'a> Validator<'a> {
    pub fn validate_for(&mut self, for_: &'a ForStmt) -> Result<(), String> {
        if let Some(init) = &for_.init {
            match init {
                VarDeclOrExpr::VarDecl(var_decl) => {
                    for decl in &var_decl.decls {
                        self.validate_pat_dup(&decl.name, var_decl.kind != VarDeclKind::Var)?;

                        if let Some(init) = &decl.init {
                            self.validate_expr(init)?;
                        }
                    }
                }
                VarDeclOrExpr::Expr(expr) => {
                    self.validate_expr(expr)?;
                }
            }
        }

        if let Some(test) = &for_.test {
            self.validate_expr(test)?;
        }

        if let Some(update) = &for_.update {
            self.validate_expr(update)?;
        }

        if single_stmt_contains_decl(&for_.body) {
            return Err(
                "Lexical declaration cannot appear in a single-statement context".to_string(),
            );
        }

        self.validate_statement(&for_.body)
    }
}
