use crate::Validator;
use swc_ecma_ast::YieldExpr;

impl<'a> Validator<'a> {
    pub fn validate_yield_expr(&mut self, yield_: &'a YieldExpr) -> Result<(), String> {
        if let Some(arg) = &yield_.arg {
            self.validate_expr(arg)?;
        }

        Ok(())
    }
}
