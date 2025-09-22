use crate::Validator;
use swc_ecma_ast::EmptyStmt;

impl Validator {
    pub fn validate_empty(_empty: &EmptyStmt) -> Result<(), String> {
        Ok(())
    }
}
