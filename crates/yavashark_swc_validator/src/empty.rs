use crate::Validator;
use swc_ecma_ast::EmptyStmt;

impl Validator<'_> {
    pub const fn validate_empty(&mut self, _empty: &EmptyStmt) -> Result<(), String> {
        Ok(())
    }
}
