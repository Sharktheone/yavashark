use crate::Validator;
use swc_ecma_ast::{ObjectLit, Prop, PropName, PropOrSpread};

impl<'a> Validator<'a> {
    pub fn validate_object_expr(&mut self, object: &'a ObjectLit) -> Result<(), String> {
        // Track if we've seen a __proto__ property definition (key: value form)
        let mut has_proto = false;

        for prop in &object.props {
            match prop {
                PropOrSpread::Prop(p) => {
                    if let Prop::KeyValue(kv) = &**p
                        && is_proto_property_name(&kv.key)
                    {
                        if has_proto {
                            return Err(
                                "Duplicate __proto__ fields are not allowed in object literals"
                                    .to_string(),
                            );
                        }
                        has_proto = true;
                    }

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
            Prop::Shorthand(ident) => {
                self.validate_ident(ident)?;
            }
            Prop::KeyValue(kv) => {
                self.validate_prop_name(&kv.key)?;
                self.validate_expr(&kv.value)?;
            }
            Prop::Assign(assign) => {
                self.validate_expr(&assign.value)?;
            }
            Prop::Getter(getter) => {
                self.validate_prop_name(&getter.key)?;
                if getter.function.is_async {
                    return Err("Getter methods cannot be async".to_string());
                }

                if getter.function.is_generator {
                    return Err("Getter methods cannot be generators".to_string());
                }

                self.validate_function(&getter.function, None, true, true)?;
            }
            Prop::Setter(setter) => {
                self.validate_prop_name(&setter.key)?;

                if setter.function.is_async {
                    return Err("Setter methods cannot be async".to_string());
                }

                if setter.function.is_generator {
                    return Err("Setter methods cannot be generators".to_string());
                }
            }
            Prop::Method(method) => {
                self.validate_prop_name(&method.key)?;

                self.validate_function(&method.function, None, true, false)?;
            }
        }

        Ok(())
    }
}

/// Check if a property name is __proto__ (as identifier or string literal)
fn is_proto_property_name(name: &PropName) -> bool {
    match name {
        PropName::Ident(ident) => ident.sym == "__proto__",
        PropName::Str(s) => s.value == "__proto__",
        _ => false,
    }
}
