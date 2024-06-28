use crate::Interpreter;
use swc_ecma_ast::{MemberExpr, MemberProp, ObjectLit};
use yavashark_env::scope::Scope;
use yavashark_env::{Context, ControlFlow, RuntimeResult, Value};

impl Interpreter {
    pub fn run_member(ctx: &mut Context, stmt: &MemberExpr, scope: &mut Scope) -> RuntimeResult {
        let obj = Self::run_expr(ctx, &stmt.obj, stmt.span, scope)?;

        let name = match &stmt.prop {
            MemberProp::Ident(i) => Value::String(i.sym.to_string()),
            MemberProp::Computed(e) => Self::run_expr(ctx, &e.expr, stmt.span, scope)?,
            MemberProp::PrivateName(_) => {
                return Err(ControlFlow::error(
                    "Unsupported member expression property".to_owned(),
                ));
            }
        };

        match obj {
            Value::Object(o) => {
                let o = o.get()?;

                Ok(o.resolve_property(&name).unwrap_or(Value::Undefined))
            }
            Value::Undefined => Err(ControlFlow::error_type(format!(
                "Cannot read property '{}' of undefined",
                name
            ))),
            Value::Null => Err(ControlFlow::error_type(format!(
                "Cannot read property '{}' of null",
                name
            ))),
            _ => Ok(Value::Undefined),
        }
    }
}
