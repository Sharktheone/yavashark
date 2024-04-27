use swc_ecma_ast::{BinaryOp, BinExpr};
use yavashark_value::error::Error;
use crate::Value;
use crate::context::Context;
use crate::RuntimeResult;
use crate::scope::Scope;

impl Context {
    pub fn run_bin(&mut self, stmt: &BinExpr, scope: &mut Scope) -> RuntimeResult {
        let left = self.run_expr(&stmt.left, stmt.span, scope)?;
        let right = self.run_expr(&stmt.right, stmt.span, scope)?;

            Ok(match stmt.op {
                BinaryOp::EqEq => { todo!() }
                BinaryOp::NotEq => { todo!() }
                BinaryOp::EqEqEq => { Value::Boolean(left == right) }
                BinaryOp::NotEqEq => { Value::Boolean(left != right) }
                BinaryOp::Lt => { Value::Boolean(left < right) }
                BinaryOp::LtEq => { Value::Boolean(left <= right) }
                BinaryOp::Gt => { Value::Boolean(left > right) }
                BinaryOp::GtEq => { Value::Boolean(left >= right) }
                BinaryOp::LShift => { left << right }
                BinaryOp::RShift => { left >> right }
                BinaryOp::ZeroFillRShift => { todo!() }
                BinaryOp::Add => { left + right }
                BinaryOp::Sub => { left - right }
                BinaryOp::Mul => { left * right }
                BinaryOp::Div => { left / right }
                BinaryOp::Mod => { left % right }
                BinaryOp::BitOr => { left | right }
                BinaryOp::BitXor => { left ^ right }
                BinaryOp::BitAnd => { left & right}
                BinaryOp::LogicalOr => { left.log_or(right) }
                BinaryOp::LogicalAnd => { left.log_and(right) }
                BinaryOp::In => { todo!() }
                BinaryOp::InstanceOf => { todo!() }
                BinaryOp::Exp => { left.pow(right) }
                BinaryOp::NullishCoalescing => { todo!() }
            })

    }
}