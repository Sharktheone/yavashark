use swc_ecma_ast::EmptyStmt;
use crate::Validator;

impl Validator {
    pub fn validate_empty(_empty: &EmptyStmt) -> Result<(), String> {
        Ok(())
    }
}
