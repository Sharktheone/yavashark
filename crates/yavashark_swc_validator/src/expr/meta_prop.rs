use swc_ecma_ast::MetaPropExpr;
use crate::Validator;

impl Validator {
    pub fn validate_meta_prop_expr(_member_prop: &MetaPropExpr) -> Result<(), String> {
        Ok(())
    }
}
