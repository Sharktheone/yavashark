use crate::Validator;
use crate::utils::single_stmt_contains_decl;
use swc_ecma_ast::{ForHead, ForOfStmt, VarDeclKind};

impl<'a> Validator<'a> {
    pub fn validate_for_of(&mut self, for_of: &'a ForOfStmt) -> Result<(), String> {
        match &for_of.left {
            ForHead::VarDecl(var_decl) => {
                for decl in &var_decl.decls {
                    if let Some(ref init) = decl.init {
                        self.validate_expr(init)?;
                    }
                    self.validate_pat_dup(&decl.name, var_decl.kind != VarDeclKind::Var)?;
                }
            }
            ForHead::UsingDecl(using_decl) => {
                for decl in &using_decl.decls {
                    if let Some(ref init) = decl.init {
                        self.validate_expr(init)?;
                    }
                    self.validate_pat(&decl.name)?;
                }
            }
            ForHead::Pat(pat) => self.validate_pat(pat)?,
        }

        self.validate_expr(&for_of.right)?;

        if single_stmt_contains_decl(&for_of.body) {
            return Err(
                "Lexical declaration cannot appear in a single-statement context".to_string(),
            );
        }

        self.validate_statement(&for_of.body)
    }
}
