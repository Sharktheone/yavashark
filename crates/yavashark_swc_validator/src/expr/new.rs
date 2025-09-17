use swc_ecma_ast::NewExpr;
use crate::Validator;

impl Validator {
    pub fn validate_new_expr(new: &NewExpr) -> Result<(), String> {
        Ok(())
    }
}
