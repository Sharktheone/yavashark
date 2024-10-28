use swc_ecma_ast::{BinExpr, BinaryOp};

use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};
use yavashark_value::Error;

use crate::Interpreter;

impl Interpreter {
    pub fn run_bin(realm: &mut Realm, stmt: &BinExpr, scope: &mut Scope) -> RuntimeResult {
        let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
        let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

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
            BinaryOp::ZeroFillRShift => left.zero_fill_rshift(&right),
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
            BinaryOp::InstanceOf => left.instance_of(&right, realm)?.into(),
            BinaryOp::Exp => left.pow(&right, realm)?,
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
