use crate::Validator;
use swc_ecma_ast::EmptyStmt;

impl<'a> Validator<'a> {
    pub fn validate_empty(&mut self, _empty: &EmptyStmt) -> Result<(), String> {
        Ok(())
    }
}
