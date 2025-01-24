use crate::Interpreter;
use swc_common::Span;
use swc_ecma_ast::{MemberExpr, MemberProp, ObjectLit};
use yavashark_env::builtins::{NumberObj, StringObj};
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_member(realm: &mut Realm, stmt: &MemberExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(realm, &stmt.obj, stmt.span, scope)?;

        Self::run_member_on(realm, value, &stmt.prop, stmt.span, scope)
    }

    pub fn run_member_on(
        realm: &mut Realm,
        value: Value,
        prop: &MemberProp,
        span: Span,
        scope: &mut Scope,
    ) -> RuntimeResult {
        Ok(Self::run_member_on_primitives(realm, value, prop, span, scope)?.0)
    }

    pub fn run_member_on_primitives(
        realm: &mut Realm,
        value: Value,
        prop: &MemberProp,
        span: Span,
        scope: &mut Scope,
    ) -> Result<(Value, Option<Value>), ControlFlow> {
        let name = match &prop {
            MemberProp::Ident(i) => Value::String(i.sym.to_string()),
            MemberProp::Computed(e) => Self::run_expr(realm, &e.expr, span, scope)?,
            MemberProp::PrivateName(_) => {
                return Err(ControlFlow::error(
                    "Unsupported member expression property".to_owned(),
                ));
            }
        };

        match value {
            Value::Object(o) => Ok((
                o.resolve_property(&name, realm)?
                    .unwrap_or(Value::Undefined),
                None,
            )),
            Value::Undefined => Err(ControlFlow::error_type(format!(
                "Cannot read property '{name}' of undefined",
            ))),
            Value::Null => Err(ControlFlow::error_type(format!(
                "Cannot read property '{name}' of null",
            ))),
            Value::String(s) => {
                let str = StringObj::with_string(realm, s)?;

                Ok((
                    str.resolve_property(&name, realm)?
                        .unwrap_or(Value::Undefined),
                    Some(str.into()),
                ))
            }

            Value::Number(n) => {
                let num = NumberObj::with_string(realm, n)?;

                Ok((
                    num.resolve_property(&name, realm)?
                        .unwrap_or(Value::Undefined),
                    Some(num.into()),
                ))
            }

            _ => Ok((Value::Undefined, None)),
        }
    }
}
