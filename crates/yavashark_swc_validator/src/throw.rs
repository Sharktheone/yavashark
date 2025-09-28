use crate::Validator;
use swc_ecma_ast::ThrowStmt;

impl<'a> Validator<'a> {
    pub fn validate_throw(&mut self, throw: &'a ThrowStmt) -> Result<(), String> {
        self.validate_expr(&throw.arg)
    }
}
