use swc_ecma_ast::AwaitExpr;
use crate::Validator;

impl Validator {
    pub fn validate_await_expr(_await: &AwaitExpr) -> Result<(), String> {
        Ok(())
    }
}
