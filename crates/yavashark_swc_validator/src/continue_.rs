use swc_ecma_ast::ContinueStmt;
use crate::Validator;

impl Validator {
    pub fn validate_continue(_cnt: &ContinueStmt) -> Result<(), String> {
        Ok(())
    }
}
