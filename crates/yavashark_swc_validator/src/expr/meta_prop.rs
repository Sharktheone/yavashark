use crate::Validator;
use swc_ecma_ast::MetaPropExpr;

impl Validator<'_> {
    pub const fn validate_meta_prop_expr(&mut self, _member_prop: &MetaPropExpr) -> Result<(), String> {
        Ok(())
    }
}
