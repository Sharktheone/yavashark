use crate::Validator;
use swc_ecma_ast::ReturnStmt;

impl Validator {
    pub fn validate_return(ret: &ReturnStmt) -> Result<(), String> {
        if let Some(arg) = &ret.arg {
            Self::validate_expr(arg)?;
        }

        Ok(())
    }
}
