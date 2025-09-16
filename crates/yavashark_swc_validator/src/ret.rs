use swc_ecma_ast::ReturnStmt;
use crate::Validator;

impl Validator {
    pub fn validate_return(ret: &ReturnStmt) -> Result<(), String> {
        if let Some(arg) = &ret.arg {
            Self::validate_expr(arg)?;
        }
        
        Ok(())
    }
}
