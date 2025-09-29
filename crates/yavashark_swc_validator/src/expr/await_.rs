use crate::Validator;
use swc_ecma_ast::AwaitExpr;

impl<'a> Validator<'a> {
    pub fn validate_await_expr(&mut self, await_: &'a AwaitExpr) -> Result<(), String> {
        if !self.in_async_function() && self.in_function_context() {
            return Err("'await' expressions are only allowed within async functions or at the top level".to_string());
        }

        self.validate_expr(&await_.arg)
    }
}
