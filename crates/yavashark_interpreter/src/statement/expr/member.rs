use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::{ControlFlow, Value};
use swc_ecma_ast::{MemberExpr, MemberProp};
use yavashark_value::error::Error;

impl Context {
    pub fn run_member(&mut self, stmt: &MemberExpr, scope: &mut Scope) -> RuntimeResult {
        let obj = self.run_expr(&stmt.obj, stmt.span, scope)?;

        let name = match &stmt.prop {
            MemberProp::Ident(i) => i.sym.to_string(),
            MemberProp::Computed(e) => {
                let value = self.run_expr(&e.expr, stmt.span, scope)?;
                value.to_string()
            }
            _ => {
                return Err(ControlFlow::error(
                    "Unsupported member expression property".to_owned(),
                ))
            }
        };

        match obj {
            Value::Object(o) => {
                let o = o.borrow();

                if let Some(v) = o.get_property(&name) {
                    Ok(v.copy())
                } else {
                    Err(ControlFlow::error(format!("Property {} not found", name)))
                }
            }
            _ => Err(ControlFlow::error(
                "Member expression object is not an object".to_owned(),
            )),
        }
    }
}
