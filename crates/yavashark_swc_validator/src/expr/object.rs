use crate::Validator;
use swc_ecma_ast::{ObjectLit, Prop, PropOrSpread};

impl<'a> Validator<'a> {
    pub fn validate_object_expr(&mut self, object: &'a ObjectLit) -> Result<(), String> {
        for prop in &object.props {
            match prop {
                PropOrSpread::Prop(p) => {
                    self.validate_prop(p)?;
                }
                PropOrSpread::Spread(spread) => {
                    self.validate_expr(&spread.expr)?;
                }
            }
        }

        Ok(())
    }

    pub fn validate_prop(&mut self, prop: &'a Prop) -> Result<(), String> {
        match prop {
            Prop::Shorthand(_) => {}
            Prop::KeyValue(kv) => {
                self.validate_prop_name(&kv.key)?;
                self.validate_expr(&kv.value)?;
            }
            Prop::Assign(assign) => {
                self.validate_expr(&assign.value)?;
            }
            Prop::Getter(getter) => {
                self.validate_prop_name(&getter.key)?;
                if let Some(stmt) = &getter.body {
                    self.validate_block(stmt)?;
                }
            }
            Prop::Setter(setter) => {
                self.validate_prop_name(&setter.key)?;

                if let Some(this_param) = &setter.this_param {
                    self.validate_pat(this_param)?;
                }

                if let Some(stmt) = &setter.body {
                    self.validate_block(stmt)?;
                }
            }
            Prop::Method(method) => {
                self.validate_prop_name(&method.key)?;

                self.validate_function(&method.function)?;
            }
        }

        Ok(())
    }
}
