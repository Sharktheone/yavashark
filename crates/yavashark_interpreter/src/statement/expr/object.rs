use swc_ecma_ast::{ObjectLit, PropOrSpread};

use crate::context::Context;
use crate::object::Object;
use crate::RuntimeResult;
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

                PropOrSpread::Prop(prop) => {}
            }
        }

        Ok(Value::Object(obj))
    }
}
