use crate::Validator;
use swc_ecma_ast::AwaitExpr;

impl<'a> Validator<'a> {
    pub fn validate_await_expr(&mut self, await_: &'a AwaitExpr) -> Result<(), String> {
        self.validate_expr(&await_.arg)
    }
}
