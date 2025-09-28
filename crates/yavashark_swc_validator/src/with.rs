use crate::Validator;
use swc_ecma_ast::WithStmt;

impl<'a> Validator<'a> {
    pub fn validate_with(&mut self, with: &'a WithStmt) -> Result<(), String> {
        self.validate_expr(&with.obj)?;
        self.validate_statement(&with.body)
    }
}
