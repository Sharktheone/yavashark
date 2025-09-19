use crate::Validator;
use swc_ecma_ast::UpdateExpr;

impl Validator {
    pub fn validate_update_expr(update: &UpdateExpr) -> Result<(), String> {
        Self::validate_expr(&update.arg)
    }
}
