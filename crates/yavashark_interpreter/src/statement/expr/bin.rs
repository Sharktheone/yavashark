use swc_ecma_ast::{BinExpr, BinaryOp, Expr, PrivateName};

use yavashark_env::scope::Scope;
use yavashark_env::{Class, ClassInstance, Realm, Res, RuntimeResult, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn run_bin(realm: &mut Realm, stmt: &BinExpr, scope: &mut Scope) -> RuntimeResult {
        Ok(match stmt.op {
            BinaryOp::EqEq => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

                Value::Boolean(left.normal_eq(&right, realm)?)
            }
            BinaryOp::NotEq => {
                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                Value::Boolean(!left.normal_eq(&right, realm)?)
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
                if let Expr::PrivateName(pn) = &*stmt.left {
                    let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                    return Ok(Self::contains_private_name(realm, pn, &right)?.into());
                }

                let left = Self::run_expr(realm, &stmt.left, stmt.span, scope)?;
                let right = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;
                right.has_key(&left, realm)?.into()
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

    pub fn contains_private_name(realm: &mut Realm, pn: &PrivateName, val: &Value) -> Res<bool> {
        Ok(val.downcast::<ClassInstance>()?.is_some_and(|c| {
            c.get_private_prop(pn.name.as_str(), realm)
                .is_ok_and(|v| v.is_some())
        }))
    }
}
