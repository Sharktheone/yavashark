use swc_ecma_ast::IfStmt;
use crate::Validator;

impl Validator {
    pub fn validate_if(_if: &IfStmt) -> Result<(), String> {
        Ok(())
    }
}
