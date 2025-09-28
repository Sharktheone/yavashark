use crate::Validator;
use swc_ecma_ast::CondExpr;

impl<'a> Validator<'a> {
    pub fn validate_cond_expr(&mut self, cond: &'a CondExpr) -> Result<(), String> {
        self.validate_expr(&cond.test)?;
        self.validate_expr(&cond.cons)?;
        self.validate_expr(&cond.alt)
    }
}
