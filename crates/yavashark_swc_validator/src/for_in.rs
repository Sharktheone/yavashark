use swc_ecma_ast::{ForHead, ForInStmt, Pat};
use crate::Validator;

impl Validator {
    pub fn validate_for_in(for_in: &ForInStmt) -> Result<(), String> {
        match &for_in.left {
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
            ForHead::Pat(pat) => {
                if matches!(&**pat, Pat::Expr(expr) if expr.is_assign() ) {
                    return Err("ForInStmt left side cannot be an expression".to_string());
                }

                Self::validate_pat(pat)?;
            },
        }
        
        
        Self::validate_expr(&for_in.right)?;
        
        Self::validate_statement(&for_in.body)
        
    }
}
