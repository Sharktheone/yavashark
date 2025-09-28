use crate::Validator;
use swc_ecma_ast::ContinueStmt;

impl<'a> Validator<'a> {
    pub fn validate_continue(&mut self, _cnt: &ContinueStmt) -> Result<(), String> {
        Ok(())
    }
}
