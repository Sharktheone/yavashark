use swc_ecma_ast::{BinExpr, BinaryOp};

use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult, Value};
use yavashark_value::Error;

use crate::Interpreter;

impl Interpreter {
    pub fn run_bin(ctx: &mut Context, stmt: &BinExpr, scope: &mut Scope) -> RuntimeResult {
        let left = Self::run_expr(ctx, &stmt.left, stmt.span, scope)?;
        let right = Self::run_expr(ctx, &stmt.right, stmt.span, scope)?;

        Ok(match stmt.op {
            BinaryOp::EqEq => Value::Boolean(left.normal_eq(&right)),
            BinaryOp::NotEq => Value::Boolean(!left.normal_eq(&right)),
            BinaryOp::EqEqEq => Value::Boolean(left == right),
            BinaryOp::NotEqEq => Value::Boolean(left != right),
            BinaryOp::Lt => Value::Boolean(left < right),
            BinaryOp::LtEq => Value::Boolean(left <= right),
            BinaryOp::Gt => Value::Boolean(left > right),
            BinaryOp::GtEq => Value::Boolean(left >= right),
            BinaryOp::LShift => left << right,
            BinaryOp::RShift => left >> right,
            BinaryOp::ZeroFillRShift => Value::Number(f64::from(
                left.to_int_or_null() as u32 >> (right.to_int_or_null() as u32 % 32),
            )),
            BinaryOp::Add => left + right,
            BinaryOp::Sub => left - right,
            BinaryOp::Mul => left * right,
            BinaryOp::Div => left / right,
            BinaryOp::Mod => left % right,
            BinaryOp::BitOr => left | right,
            BinaryOp::BitXor => left ^ right,
            BinaryOp::BitAnd => left & right,
            BinaryOp::LogicalOr => left.log_or(right),
            BinaryOp::LogicalAnd => left.log_and(right),
            BinaryOp::In => right.contains_key(&left)?.into(),
            BinaryOp::InstanceOf => {
                let Value::Object(obj) = left else {
                    return Ok(Value::Boolean(false));
                };

                let Value::Object(constructor) = right else {
                    return Err(
                        Error::ty("Right-hand side of 'instanceof' is not an object").into(),
                    );
                };

                let Value::Object(constructor) = constructor.get_constructor_value(ctx).ok_or(
                    Error::ty("Right-hand side of 'instanceof' is not a constructor"),
                )?
                else {
                    return Err(Error::ty(
                        "Right-hand side of 'instanceof' has not an object as constructor",
                    )
                    .into());
                };

                let constructor_proto = constructor.get()?.prototype();

                let mut proto = Some(obj.get()?.prototype());

                while let Some(mut p) = proto {
                    if p == constructor_proto {
                        return Ok(Value::Boolean(true));
                    }

                    if let Value::Object(o) = p {
                        proto = Some(o.get()?.prototype());
                    } else {
                        break;
                    }
                }

                return Ok(Value::Boolean(false));
            }
            BinaryOp::Exp => left.pow(&right),
            BinaryOp::NullishCoalescing => {
                if left.is_nullish() {
                    right
                } else {
                    left
                }
            }
        })
    }
}
