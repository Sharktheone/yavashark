use swc_ecma_ast::BreakStmt;
use crate::Validator;

impl Validator {
    pub fn validate_break(_brk: &BreakStmt) -> Result<(), String> {
        Ok(())
    }
}
