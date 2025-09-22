use crate::Validator;
use swc_ecma_ast::{ObjectLit, Prop, PropOrSpread};

impl Validator {
    pub fn validate_object_expr(object: &ObjectLit) -> Result<(), String> {
        for prop in &object.props {
            match prop {
                PropOrSpread::Prop(p) => {
                    Self::validate_prop(p)?;
                }
                PropOrSpread::Spread(spread) => {
                    Self::validate_expr(&spread.expr)?;
                }
            }
        }

        Ok(())
    }

    pub fn validate_prop(prop: &Prop) -> Result<(), String> {
        match prop {
            Prop::Shorthand(_) => {}
            Prop::KeyValue(kv) => {
                Self::validate_prop_name(&kv.key)?;
                Self::validate_expr(&kv.value)?;
            }
            Prop::Assign(assign) => {
                Self::validate_expr(&assign.value)?;
            }
            Prop::Getter(getter) => {
                Self::validate_prop_name(&getter.key)?;
                if let Some(stmt) = &getter.body {
                    Self::validate_block(stmt)?;
                }
            }
            Prop::Setter(setter) => {
                Self::validate_prop_name(&setter.key)?;

                if let Some(this_param) = &setter.this_param {
                    Self::validate_pat(this_param)?;
                }

                if let Some(stmt) = &setter.body {
                    Self::validate_block(stmt)?;
                }
            }
            Prop::Method(method) => {
                Self::validate_prop_name(&method.key)?;

                Self::validate_function(&method.function)?;
            }
        }

        Ok(())
    }
}
