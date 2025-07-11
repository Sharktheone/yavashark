use crate::Interpreter;
use swc_ecma_ast::{Expr, UnaryExpr, UnaryOp};
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, RuntimeResult, Value};
use yavashark_string::YSString;

impl Interpreter {
    pub fn run_unary(realm: &mut Realm, stmt: &UnaryExpr, scope: &mut Scope) -> RuntimeResult {
        if stmt.op == UnaryOp::Delete {
            match &*stmt.arg {
                Expr::Ident(i) => {
                    return Ok(scope.resolve(i.sym.as_str())?.is_none().into());
                }
                Expr::Member(m) => {
                    let obj = Self::run_expr(realm, &m.obj, m.span, scope)?;
                    if let Value::Object(obj) = obj {
                        let name = match &m.prop {
                            swc_ecma_ast::MemberProp::Ident(i) => {
                                Value::String(YSString::from_ref(&i.sym))
                            }
                            swc_ecma_ast::MemberProp::PrivateName(p) => {
                                Value::String(YSString::from_ref(&p.name))
                            }
                            swc_ecma_ast::MemberProp::Computed(c) => {
                                Self::run_expr(realm, &c.expr, c.span, scope)?
                            }
                        };

                        return Ok(obj.delete_property(&name)?.is_some().into());
                    }
                }
                Expr::Call(call) => {
                    Self::run_call(realm, call, scope)?;

                    return Ok(true.into());
                }
                Expr::SuperProp(sp) => {
                    let this = scope.this()?;
                    let proto = this.prototype(realm)?;
                    let sup = proto.prototype(realm)?;

                    if sup.is_null() {
                        return Err(Error::reference(
                            "Cannot delete property of null or undefined",
                        )
                        .into());
                    }

                    let sup = sup.as_object()?;

                    return match &sp.prop {
                        swc_ecma_ast::SuperProp::Ident(i) => {
                            let name = i.sym.to_string();
                            Ok(sup.delete_property(&name.into())?.is_some().into())
                        }
                        swc_ecma_ast::SuperProp::Computed(p) => {
                            let name = Self::run_expr(realm, &p.expr, p.span, scope)?;
                            Ok(sup.delete_property(&name)?.is_some().into())
                        }
                    };
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
                    (-(&*b)).into()
                } else {
                    Value::Number(-value.to_number(realm)?)
                }
            }
            UnaryOp::Bang => Value::Boolean(!value.is_truthy()),
            UnaryOp::Tilde => Value::Number((!(value.to_int_or_null(realm))?) as f64),
            UnaryOp::TypeOf => Value::String(value.type_of().into()),
            UnaryOp::Void => Value::Undefined,
            UnaryOp::Delete => Value::Boolean(false), // unreachable
        })
    }
}
