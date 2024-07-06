use crate::Interpreter;
use swc_common::Span;
use swc_ecma_ast::{MemberExpr, MemberProp, ObjectLit};
use yavashark_env::scope::Scope;
use yavashark_env::{Context, ControlFlow, RuntimeResult, Value};

impl Interpreter {
    pub fn run_member(ctx: &mut Context, stmt: &MemberExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(ctx, &stmt.obj, stmt.span, scope)?;

        Self::run_member_on(ctx, value, &stmt.prop, stmt.span, scope)
    }

    pub fn run_member_on(
        ctx: &mut Context,
        value: Value,
        prop: &MemberProp,
        span: Span,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let name = match &prop {
            MemberProp::Ident(i) => Value::String(i.sym.to_string()),
            MemberProp::Computed(e) => Self::run_expr(ctx, &e.expr, span, scope)?,
            MemberProp::PrivateName(_) => {
                return Err(ControlFlow::error(
                    "Unsupported member expression property".to_owned(),
                ));
            }
        };

        match value {
            Value::Object(o) => Ok(o.resolve_property(&name, ctx)?.unwrap_or(Value::Undefined)),
            Value::Undefined => Err(ControlFlow::error_type(format!(
                "Cannot read property '{name}' of undefined",
            ))),
            Value::Null => Err(ControlFlow::error_type(format!(
                "Cannot read property '{name}' of null",
            ))),
            _ => Ok(Value::Undefined),
        }
    }
}
