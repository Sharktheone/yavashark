use swc_ecma_ast::{
    AssignExpr, AssignOp, AssignTarget, MemberExpr, MemberProp, SimpleAssignTarget,
};

use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, RuntimeResult, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn run_assign(realm: &mut Realm, stmt: &AssignExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(realm, &stmt.right, stmt.span, scope)?;

        if stmt.op == AssignOp::Assign {
            return Ok(
                Self::assign_target(realm, &stmt.left, value, scope).map(|()| Value::Undefined)?
            );
        }

        Self::assign_target_op(realm, stmt.op, &stmt.left, value, scope)
    }

    pub fn assign_target(
        realm: &mut Realm,
        target: &AssignTarget,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
        match target {
            AssignTarget::Simple(t) => match t {
                SimpleAssignTarget::Ident(i) => {
                    let name = i.sym.to_string();
                    scope.update_or_define(name, value)
                }
                SimpleAssignTarget::Member(m) => Self::assign_member(realm, m, value, scope),
                _ => todo!("assign targets"),
            },
            AssignTarget::Pat(_) => {
                todo!("Pattern assignment")
            }
        }
    }

    pub fn assign_member(
        realm: &mut Realm,
        m: &MemberExpr,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
        let obj = Self::run_expr(realm, &m.obj, m.span, scope)?;
        if let Value::Object(obj) = obj {
            let name = match &m.prop {
                MemberProp::Ident(i) => Value::String(i.sym.to_string()),
                MemberProp::PrivateName(p) => Value::String(p.name.to_string()),
                MemberProp::Computed(c) => Self::run_expr(realm, &c.expr, c.span, scope)?,
            };

            obj.define_property(name, value);
            Ok(())
        } else {
            Err(Error::ty("Invalid left-hand side in assignment"))
        }
    }

    pub fn assign_target_op(
        realm: &mut Realm,
        op: AssignOp,
        target: &AssignTarget,
        left: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        match target {
            AssignTarget::Simple(t) => match t {
                SimpleAssignTarget::Ident(i) => {
                    let name = i.sym.to_string();

                    let right = scope
                        .resolve(&name, realm)?
                        .ok_or_else(|| Error::reference_error(format!("{name} is not defined")))?;

                    let value = Self::run_assign_op(op, left, right, realm)?;

                    scope.update(&name, value.copy())?;

                    Ok(value)
                }
                SimpleAssignTarget::Member(m) => Self::assign_member_op(realm, op, m, left, scope),
                _ => todo!("assign targets"),
            },
            AssignTarget::Pat(_) => {
                todo!("Pattern assignment")
            }
        }
    }

    pub fn assign_member_op(
        realm: &mut Realm,
        op: AssignOp,
        m: &MemberExpr,
        left: Value,
        scope: &mut Scope,
    ) -> RuntimeResult {
        let obj = Self::run_expr(realm, &m.obj, m.span, scope)?;
        if let Value::Object(obj) = obj {
            let name = match &m.prop {
                MemberProp::Ident(i) => Value::String(i.sym.to_string()),
                MemberProp::PrivateName(p) => Value::String(p.name.to_string()),
                MemberProp::Computed(c) => Self::run_expr(realm, &c.expr, c.span, scope)?,
            };

            let right = obj
                .resolve_property(&name, realm)?
                .unwrap_or(Value::Undefined);

            let value = Self::run_assign_op(op, left, right, realm)?;

            obj.define_property(name, value.copy());
            Ok(value)
        } else {
            Err(Error::ty("Invalid left-hand side in assignment").into())
        }
    }

    pub fn run_assign_op(
        op: AssignOp,
        left: Value,
        right: Value,
        realm: &mut Realm,
    ) -> RuntimeResult {
        Ok(match op {
            AssignOp::Assign => right,
            AssignOp::AddAssign => left + right,
            AssignOp::SubAssign => left - right,
            AssignOp::MulAssign => left * right,
            AssignOp::DivAssign => left / right,
            AssignOp::ModAssign => left % right,
            AssignOp::LShiftAssign => left << right,
            AssignOp::RShiftAssign => left >> right,
            AssignOp::ZeroFillRShiftAssign => left.zero_fill_rshift(&right),
            AssignOp::BitOrAssign => left | right,
            AssignOp::BitXorAssign => left ^ right,
            AssignOp::BitAndAssign => left & right,
            AssignOp::ExpAssign => left.pow(&right, realm)?,
            AssignOp::AndAssign => left.log_and(right),
            AssignOp::OrAssign => left.log_or(right),
            AssignOp::NullishAssign => {
                if left.is_nullish() {
                    right
                } else {
                    left
                }
            }
        })
    }
}
