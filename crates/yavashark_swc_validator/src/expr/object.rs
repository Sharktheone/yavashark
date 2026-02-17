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
                        && is_proto_property_name(&kv.key) {
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
                if let Some(body) = &getter.body {
                    let scope = self.enter_function_context(false, false);
                    self.set_super_property_allowed(true);
                    self.set_super_call_allowed(true);
                    let super_prop_guard = self.enter_super_property_scope();
                    let super_call_guard = self.enter_super_call_scope();

                    let result = self.validate_block(body);

                    super_call_guard.exit(self);
                    super_prop_guard.exit(self);
                    scope.exit(self);
                    result?;
                }
            }
            Prop::Setter(setter) => {
                self.validate_prop_name(&setter.key)?;

                if let Some(this_param) = &setter.this_param {
                    self.validate_pat(this_param)?;
                }

                if let Some(body) = &setter.body {
                    let scope = self.enter_function_context(false, false);

                    if crate::utils::block_has_use_strict(body) {
                        self.set_current_function_strict();
                    }

                    self.validate_pat(&setter.param)?;

                    self.set_super_property_allowed(true);
                    self.set_super_call_allowed(true);
                    let super_prop_guard = self.enter_super_property_scope();
                    let super_call_guard = self.enter_super_call_scope();

                    let result = self.validate_block(body);

                    super_call_guard.exit(self);
                    super_prop_guard.exit(self);
                    scope.exit(self);
                    result?;
                } else {
                    self.validate_pat(&setter.param)?;
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
