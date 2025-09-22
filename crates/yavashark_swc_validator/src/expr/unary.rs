use crate::Validator;
use swc_ecma_ast::UnaryExpr;

impl Validator {
    pub fn validate_unary_expr(unary: &UnaryExpr) -> Result<(), String> {
        Self::validate_expr(&unary.arg)
    }
}
