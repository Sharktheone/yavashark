use crate::Validator;
use swc_ecma_ast::{SuperProp, SuperPropExpr};

impl<'a> Validator<'a> {
    pub fn validate_super_prop_expr(&mut self, super_prop: &'a SuperPropExpr) -> Result<(), String> {
        if let SuperProp::Computed(c) = &super_prop.prop {
            self.validate_expr(&c.expr)?;
        }

        Ok(())
    }
}
