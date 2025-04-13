use crate::Interpreter;
use swc_ecma_ast::{Expr, UnaryExpr, UnaryOp};
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_unary(realm: &mut Realm, stmt: &UnaryExpr, scope: &mut Scope) -> RuntimeResult {
        if stmt.op == UnaryOp::Delete {
            match &*stmt.arg {
                Expr::Ident(i) => return Ok(false.into()),
                Expr::Member(m) => {
                    let obj = Self::run_expr(realm, &m.obj, m.span, scope)?;
                    if let Value::Object(obj) = obj {
                        let name = match &m.prop {
                            swc_ecma_ast::MemberProp::Ident(i) => Value::String(i.sym.to_string()),
                            swc_ecma_ast::MemberProp::PrivateName(p) => {
                                Value::String(p.name.to_string())
                            }
                            swc_ecma_ast::MemberProp::Computed(c) => {
                                Self::run_expr(realm, &c.expr, c.span, scope)?
                            }
                        };

                        return Ok(obj.delete_property(&name)?.is_some().into());
                    }
                }
                _ => {}
            }
        }

        let value = Self::run_expr(realm, &stmt.arg, stmt.span, scope).or_else(|v| {
            if stmt.op == UnaryOp::TypeOf {
                Ok(Value::Undefined)
            } else {
                Err(v)
            }
        })?;

        Ok(match stmt.op {
            UnaryOp::Plus => Value::Number(value.to_number(realm)?),
            UnaryOp::Minus => {
                if let Value::BigInt(b) = value {
                    Value::BigInt(-b)
                } else {
                    Value::Number(-value.to_number(realm)?)
                }
            },
            UnaryOp::Bang => Value::Boolean(!value.is_truthy()),
            UnaryOp::Tilde => Value::Number((!(value.to_int_or_null())?) as f64),
            UnaryOp::TypeOf => Value::String(value.type_of().into()),
            UnaryOp::Void => Value::Undefined,
            UnaryOp::Delete => Value::Boolean(false), // unreachable
        })
    }
}
