use crate::Interpreter;
use swc_common::Span;
use swc_ecma_ast::{MemberExpr, MemberProp, ObjectLit};
use yavashark_env::builtins::{BigIntObj, BooleanObj, NumberObj, StringObj, SymbolObj};
use yavashark_env::scope::Scope;
use yavashark_env::{Class, ClassInstance, ControlFlow, Error, Realm, RuntimeResult, Value};
use yavashark_string::YSString;
use yavashark_value::Obj;

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
            MemberProp::Ident(i) => Value::String(YSString::from_ref(&i.sym)),
            MemberProp::Computed(e) => Self::run_expr(realm, &e.expr, span, scope)?,
            MemberProp::PrivateName(p) => {
                let name = p.name.as_str();
                let obj = value.as_object()?;

                if let Some(class) = obj.downcast::<ClassInstance>() {
                    let val = class
                        .get_private_prop(name)?
                        .ok_or(Error::ty_error(format!("Private name {name} not found")))?;

                    return Ok((val, None));
                };

                if let Some(class) = obj.downcast::<Class>() {
                    let val = class
                        .get_private_prop(name)
                        .ok_or(Error::ty_error(format!("Private name {name} not found")))?;

                    return Ok((val.copy(), None));
                }

                return Err(ControlFlow::error_type(format!(
                    "Private name {name} can only be used in class"
                )));
            }
        };

        match value {
            Value::Object(ref o) => Ok((
                o.resolve_property(&name, realm)?
                    .unwrap_or(Value::Undefined),
                Some(value),
            )),
            Value::Undefined => Err(ControlFlow::error_type(format!(
                "Cannot read property '{name}' of undefined",
            ))),
            Value::Null => Err(ControlFlow::error_type(format!(
                "Cannot read property '{name}' of null",
            ))),
            Value::String(s) => {
                let str = Obj::into_object(StringObj::with_string(realm, s));

                Ok((
                    str.resolve_property(&name, realm)?
                        .unwrap_or(Value::Undefined),
                    Some(str.into()),
                ))
            }

            Value::Number(n) => {
                let num = NumberObj::with_number(realm, n)?;

                Ok((
                    num.resolve_property(&name, realm)?
                        .unwrap_or(Value::Undefined),
                    Some(num.into()),
                ))
            }

            Value::Boolean(b) => {
                let boolean = BooleanObj::new(realm, b);

                Ok((
                    boolean
                        .resolve_property(&name, realm)?
                        .unwrap_or(Value::Undefined),
                    Some(boolean.into()),
                ))
            }

            Value::Symbol(s) => {
                let symbol = SymbolObj::new(realm, s);

                Ok((
                    symbol
                        .resolve_property(&name, realm)?
                        .unwrap_or(Value::Undefined),
                    Some(symbol.into()),
                ))
            }

            Value::BigInt(big_int) => {
                let big_int = BigIntObj::new(realm, big_int);

                Ok((
                    big_int
                        .resolve_property(&name, realm)?
                        .unwrap_or(Value::Undefined),
                    Some(big_int.into()),
                ))
            }
        }
    }
}
