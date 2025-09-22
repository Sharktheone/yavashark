use crate::Validator;
use swc_ecma_ast::BreakStmt;

impl Validator {
    pub fn validate_break(_brk: &BreakStmt) -> Result<(), String> {
        Ok(())
    }
}
