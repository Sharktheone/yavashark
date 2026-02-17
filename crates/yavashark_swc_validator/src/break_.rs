use crate::Validator;
use swc_ecma_ast::BreakStmt;

impl Validator<'_> {
    pub const fn validate_break(&mut self, _brk: &BreakStmt) -> Result<(), String> {
        Ok(())
    }
}
