use swc_ecma_ast::ForStmt;
use crate::Validator;

impl Validator {
    pub fn validate_for(_for: &ForStmt) -> Result<(), String> {
        Ok(())
    }
}
