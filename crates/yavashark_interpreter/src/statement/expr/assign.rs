use swc_ecma_ast::{AssignExpr, AssignTarget, MemberExpr, MemberProp, SimpleAssignTarget};

use yavashark_env::scope::Scope;
use yavashark_env::{Context, Error, Res, RuntimeResult, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn run_assign(ctx: &mut Context, stmt: &AssignExpr, scope: &mut Scope) -> RuntimeResult {
        let value = Self::run_expr(ctx, &stmt.right, stmt.span, scope)?;

        Ok(Self::assign_target(ctx, &stmt.left, value, scope).map(|()| Value::Undefined)?)
    }

    pub fn assign_target(
        ctx: &mut Context,
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
                SimpleAssignTarget::Member(m) => Self::assign_member(ctx, m, value, scope),
                _ => todo!("assign targets"),
            },
            AssignTarget::Pat(_) => {
                todo!("Pattern assignment")
            }
        }
    }

    pub fn assign_member(
        ctx: &mut Context,
        m: &MemberExpr,
        value: Value,
        scope: &mut Scope,
    ) -> Res {
        let obj = Self::run_expr(ctx, &m.obj, m.span, scope)?;
        if let Value::Object(obj) = obj {
            let name = match &m.prop {
                MemberProp::Ident(i) => Value::String(i.sym.to_string()),
                MemberProp::PrivateName(p) => Value::String(p.id.sym.to_string()),
                MemberProp::Computed(c) => Self::run_expr(ctx, &c.expr, c.span, scope)?,
            };

            obj.define_property(name, value);
            Ok(())
        } else {
            Err(Error::ty("Invalid left-hand side in assignment"))
        }
    }
}
