use crate::Validator;
use swc_ecma_ast::BreakStmt;

impl<'a> Validator<'a> {
    pub fn validate_break(&mut self, _brk: &BreakStmt) -> Result<(), String> {
        Ok(())
    }
}
