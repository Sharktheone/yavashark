use swc_ecma_ast::AwaitExpr;
use crate::Validator;

impl Validator {
    pub fn validate_await_expr(await_: &AwaitExpr) -> Result<(), String> {
        Self::validate_expr(&await_.arg)
    }
}
