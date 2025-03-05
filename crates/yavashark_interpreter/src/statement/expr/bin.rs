use swc_ecma_ast::{BinExpr, BinaryOp};

use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn run_bin(realm: &mut Realm, stmt: &BinExpr, scope: &mut Scope) -> RuntimeResult {
        Ok(match stmt.op {
            BinaryOp::EqEq => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

                Value::Boolean(left.normal_eq(&right))
            }
            BinaryOp::NotEq => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                Value::Boolean(!left.normal_eq(&right))
            }
            BinaryOp::EqEqEq => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                Value::Boolean(left == right)
            }
            BinaryOp::NotEqEq => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                Value::Boolean(left != right)
            }
            BinaryOp::Lt => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                Value::Boolean(left < right)
            }
            BinaryOp::LtEq => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                Value::Boolean(left <= right)
            }
            BinaryOp::Gt => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                Value::Boolean(left > right)
            }
            BinaryOp::GtEq => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                Value::Boolean(left >= right)
            }
            BinaryOp::LShift => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.shl(&right, realm)?
            }
            BinaryOp::RShift => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.shr(&right, realm)?
            }
            BinaryOp::ZeroFillRShift => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.ushr(&right, realm)?
            }
            BinaryOp::Add => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.add(&right, realm)?
            }
            BinaryOp::Sub => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.sub(&right, realm)?
            }
            BinaryOp::Mul => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.mul(&right, realm)?
            }
            BinaryOp::Div => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.div(&right, realm)?
            }
            BinaryOp::Mod => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.rem(&right, realm)?
            }
            BinaryOp::BitOr => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.or(&right, realm)?
            }
            BinaryOp::BitXor => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.xor(&right, realm)?
            }
            BinaryOp::BitAnd => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.and(&right, realm)?
            }
            BinaryOp::LogicalOr => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;

                if left.is_truthy() {
                    return Ok(left);
                }

                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

                return Ok(right);
            }
            BinaryOp::LogicalAnd => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                if left.is_falsey() {
                    return Ok(left);
                }
                
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                
                return Ok(right);
            }
            BinaryOp::In => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                right.contains_key(&left)?.into()
            }
            BinaryOp::InstanceOf => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.instance_of(&right, realm)?.into()
            }
            BinaryOp::Exp => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                left.exp(&right, realm)?
            }
            BinaryOp::NullishCoalescing => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                if left.is_nullish() {
                    right
                } else {
                    left
                }
            }
        })
    }
}
