use crate::Interpreter;
use swc_ecma_ast::{UnaryExpr, UnaryOp};
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult, Value};

impl Interpreter {
    pub fn run_unary(ctx: &mut Context, stmt: &UnaryExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(ctx, &stmt.arg, stmt.span, scope).or_else(|v| {
            if stmt.op == UnaryOp::TypeOf {
                Ok(Value::String("undefined".to_string()))
            } else {
                Err(v)
            }
        })?;

        Ok(match stmt.op {
            UnaryOp::Plus => Value::Number(value.to_number(ctx)?),
            UnaryOp::Minus => Value::Number(-value.to_number(ctx)?),
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
