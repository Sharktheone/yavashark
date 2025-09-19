use crate::Validator;
use swc_ecma_ast::ContinueStmt;

impl Validator {
    pub fn validate_continue(_cnt: &ContinueStmt) -> Result<(), String> {
        Ok(())
    }
}
