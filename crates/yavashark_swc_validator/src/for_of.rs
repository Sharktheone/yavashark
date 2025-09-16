use swc_ecma_ast::{ForHead, ForOfStmt};
use crate::Validator;

impl Validator {
    pub fn validate_for_of(for_of: &ForOfStmt) -> Result<(), String> {
        match &for_of.left {
            ForHead::VarDecl(var_decl) => {
                for decl in &var_decl.decls {
                    if let Some(ref init) = decl.init {
                        Self::validate_expr(init)?;
                    }
                    Self::validate_pat(&decl.name)?;
                }
            }
            ForHead::UsingDecl(using_decl) => {
                for decl in &using_decl.decls {
                    if let Some(ref init) = decl.init {
                        Self::validate_expr(init)?;
                    }
                    Self::validate_pat(&decl.name)?;
                }
            }
            ForHead::Pat(pat) => Self::validate_pat(pat)?,
        }


        Self::validate_expr(&for_of.right)?;

        Self::validate_statement(&for_of.body)

    }
}
