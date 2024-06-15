use swc_ecma_ast::{Expr, UpdateExpr, UpdateOp};
use yavashark_env::{Context, Error, RuntimeResult, Value};
use yavashark_env::scope::Scope;
use crate::Interpreter;


impl Interpreter {
    pub fn run_update(ctx: &mut Context, stmt: &UpdateExpr, scope: &mut Scope) -> RuntimeResult {
        fn update(value: Value, op: UpdateOp) -> (Value, Value) {
            match op {
                UpdateOp::PlusPlus => (
                    value.copy() + Value::Number(1.0),
                    value + Value::Number(0.0),
                ),
                UpdateOp::MinusMinus => (
                    value.copy() - Value::Number(1.0),
                    value + Value::Number(0.0),
                ),
            }
        }

        match &*stmt.arg {
            Expr::Ident(i) => {
                let name = i.sym.to_string();
                let value = scope
                    .resolve(&name)?
                    .ok_or(Error::reference_error(format!("{name} is not defined")))?;
                let up = update(value, stmt.op);
                scope.update_or_define(name, up.0);
                Ok(up.1)
            }
            Expr::Member(m) => {
                let value = Self::run_member(ctx, m, scope)?;

                let up = update(value, stmt.op);

                Self::assign_member(ctx, m, up.0, scope);
                Ok(up.1)
            }

            e => {
                let value = Self::run_expr(ctx, e, stmt.span, scope)?;
                //TODO: this isn't correct
                match stmt.op {
                    UpdateOp::PlusPlus => {
                        let value = value + Value::Number(1.0);
                        Ok(value)
                    }
                    UpdateOp::MinusMinus => {
                        let value = value - Value::Number(1.0);
                        Ok(value)
                    }
                }
            }
        }
    }
}
