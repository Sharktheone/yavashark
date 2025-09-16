use swc_ecma_ast::BreakStmt;
use crate::Validator;

impl Validator {
    pub fn validate_break(brk: &BreakStmt) -> Result<(), String> {
        Ok(())
    }
}
