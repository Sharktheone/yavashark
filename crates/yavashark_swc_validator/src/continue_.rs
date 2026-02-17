use crate::Validator;
use swc_ecma_ast::ContinueStmt;

impl Validator<'_> {
    pub const fn validate_continue(&mut self, _cnt: &ContinueStmt) -> Result<(), String> {
        Ok(())
    }
}
