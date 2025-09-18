use swc_ecma_ast::{SuperProp, SuperPropExpr};
use crate::Validator;

impl Validator {
    pub fn validate_super_prop_expr(super_prop: &SuperPropExpr) -> Result<(), String> {
        if let SuperProp::Computed(c) = &super_prop.prop {
            Self::validate_expr(&c.expr)?;
        }

        Ok(())
    }
}
