use crate::Interpreter;
use swc_ecma_ast::{MemberExpr, MemberProp};
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

                o.resolve_property(&name).ok_or_else(|| {
                    ControlFlow::error(format!("Property {name:?} not found"))
                })
            }
            _ => Err(ControlFlow::error(
                format!("Cannot read property {name:?} of {obj:?}"), //TODO: convert to the non-primitive form of the value and try to access the property there
            )),
        }
    }
}
