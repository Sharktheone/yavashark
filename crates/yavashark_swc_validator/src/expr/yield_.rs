use swc_ecma_ast::YieldExpr;
use crate::Validator;

impl Validator {
    pub fn validate_yield_expr(_yield: &YieldExpr) -> Result<(), String> {
        Ok(())
    }
}
