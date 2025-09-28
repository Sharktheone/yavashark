use crate::Validator;
use swc_ecma_ast::UpdateExpr;

impl<'a> Validator<'a> {
    pub fn validate_update_expr(&mut self, update: &'a UpdateExpr) -> Result<(), String> {
        self.ensure_valid_assignment_target_expr(&update.arg)
    }
}
