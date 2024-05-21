use swc_common::Spanned;
use swc_ecma_ast::{ObjectLit, Prop, PropName, PropOrSpread};

use crate::{ControlFlow, RuntimeResult};
use crate::context::Context;
use crate::object::Object;
use crate::scope::Scope;
use crate::Value;

impl Context {
    pub fn run_object(&mut self, stmt: &ObjectLit, scope: &mut Scope) -> RuntimeResult {
        let mut obj = Object::new(self);

        for prop in &stmt.props {
            match prop {
                PropOrSpread::Spread(spread) => {
                    let expr = self.run_expr(&spread.expr, spread.dot3_token, scope)?;

                    if let Ok(props) = expr.properties() {
                        for (name, value) in props {
                            obj.define_property(name, value);
                        }
                    }
                }

                PropOrSpread::Prop(prop) => {
                    match &**prop {
                        Prop::Shorthand(ident) => {
                            let name = ident.sym.to_string();
                            let value = scope.resolve(&name).ok_or(ControlFlow::error_reference(format!(
                                "{name} is not defined"
                            )))?;

                            obj.define_property(name.into(), value);
                        }
                        Prop::KeyValue(kv) => {
                            let key = match &kv.key {
                                PropName::Ident(ident) => Value::String(ident.sym.to_string()),
                                PropName::Str(str_) => Value::String(str_.value.to_string()),
                                PropName::Num(num) => Value::Number(num.value),
                                PropName::Computed(expr) => self.run_expr(&expr.expr, expr.span, scope)?,
                                PropName::BigInt(_) => todo!(),
                            };


                            let value = self.run_expr(&kv.value, prop.span(), scope)?;

                            obj.define_property(key, value);
                        }
                        Prop::Assign(assign) => {
                            let key = assign.key.sym.to_string();

                            let value = self.run_expr(&assign.value, prop.span(), scope)?;
                            
                            obj.define_property(key.into(), value);
                        }
                        _ => todo!(),
        
                    }
                }
            }
        }

        Ok(Value::Object(obj))
    }
}
