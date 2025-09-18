use swc_ecma_ast::UnaryExpr;
use crate::Validator;

impl Validator {
    pub fn validate_unary_expr(unary: &UnaryExpr) -> Result<(), String> {
        Self::validate_expr(&unary.arg)
    }
}
