use crate::Interpreter;
use swc_ecma_ast::{UnaryExpr, UnaryOp};
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_unary(realm: &mut Realm, stmt: &UnaryExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(realm, &stmt.arg, stmt.span, scope).or_else(|v| {
            if stmt.op == UnaryOp::TypeOf {
                Ok(Value::String("undefined".to_string()))
            } else {
                Err(v)
            }
        })?;

        Ok(match stmt.op {
            UnaryOp::Plus => Value::Number(value.to_number(realm)?),
            UnaryOp::Minus => Value::Number(-value.to_number(realm)?),
            UnaryOp::Bang => Value::Boolean(!value.is_truthy()),
            UnaryOp::Tilde => Value::Number((!(value.to_int_or_null())) as f64),
            UnaryOp::TypeOf => Value::String(value.type_of().into()),
            UnaryOp::Void => Value::Undefined,
            UnaryOp::Delete => {
                todo!()
            }
        })
    }
}
