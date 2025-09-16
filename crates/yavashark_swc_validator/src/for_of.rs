use swc_ecma_ast::ForOfStmt;
use crate::Validator;

impl Validator {
    pub fn validate_for_of(_for_of: &ForOfStmt) -> Result<(), String> {
        Ok(())
    }
}
