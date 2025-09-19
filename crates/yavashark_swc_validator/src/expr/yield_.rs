use crate::Validator;
use swc_ecma_ast::YieldExpr;

impl Validator {
    pub fn validate_yield_expr(yield_: &YieldExpr) -> Result<(), String> {
        if let Some(arg) = &yield_.arg {
            Self::validate_expr(arg)?;
        }

        Ok(())
    }
}
