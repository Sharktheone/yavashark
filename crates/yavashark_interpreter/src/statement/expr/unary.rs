use swc_ecma_ast::{UnaryExpr, UnaryOp};

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;

impl Context {
    pub fn run_unary(&mut self, stmt: &UnaryExpr, scope: &mut Scope) -> RuntimeResult {
        let value = self.run_expr(&stmt.arg, stmt.span, scope).or_else(|v| {
            if stmt.op == UnaryOp::TypeOf {
                Ok(Value::String("undefined".to_string()))
            } else {
                Err(v)
            }
        })?;

        Ok(match stmt.op {
            UnaryOp::Plus => Value::Number(value.to_number_or_null()),
            UnaryOp::Minus => Value::Number(-value.to_number_or_null()),
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
