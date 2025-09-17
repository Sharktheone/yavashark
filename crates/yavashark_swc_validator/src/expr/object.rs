use swc_ecma_ast::{ObjectLit, PropOrSpread};
use crate::Validator;

impl Validator {
    pub fn validate_object_expr(object: &ObjectLit) -> Result<(), String> {
        for prop in &object.props {
            match prop {
                PropOrSpread::Prop(_p) => {
                    //TODO
                }
                PropOrSpread::Spread(spread) => {
                    Self::validate_expr(&spread.expr)?;
                }
            }
        }

        Ok(())
    }
}
