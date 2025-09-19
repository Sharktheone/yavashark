use crate::Validator;
use swc_ecma_ast::MetaPropExpr;

impl Validator {
    pub fn validate_meta_prop_expr(_member_prop: &MetaPropExpr) -> Result<(), String> {
        Ok(())
    }
}
