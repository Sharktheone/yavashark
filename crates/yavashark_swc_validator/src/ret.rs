use swc_ecma_ast::ReturnStmt;
use crate::Validator;

impl Validator {
    pub fn validate_return(ret: &ReturnStmt) -> Result<(), String> {
        Ok(())
    }
}
