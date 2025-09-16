use swc_ecma_ast::ReturnStmt;
use crate::Validator;

impl Validator {
    pub fn validate_return(_ret: &ReturnStmt) -> Result<(), String> {
        Ok(())
    }
}
