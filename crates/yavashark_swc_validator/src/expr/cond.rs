use swc_ecma_ast::CondExpr;
use crate::Validator;

impl Validator {
    pub fn validate_cond_expr(cond: &CondExpr) -> Result<(), String> {
        Self::validate_expr(&cond.test)?;
        Self::validate_expr(&cond.cons)?;
        Self::validate_expr(&cond.alt)
    }
}
