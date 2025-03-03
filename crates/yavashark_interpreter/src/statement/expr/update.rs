use crate::Interpreter;
use swc_ecma_ast::{Expr, UpdateExpr, UpdateOp};
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_update(realm: &mut Realm, stmt: &UpdateExpr, scope: &mut Scope) -> RuntimeResult {
        fn update(value: Value, op: UpdateOp) -> (Value, Value) {
            match op {
                UpdateOp::PlusPlus => (
                    value.copy() - Value::Number(-1.0),
                    value - Value::Number(0.0),
                ),
                UpdateOp::MinusMinus => (
                    value.copy() - Value::Number(1.0),
                    value - Value::Number(0.0),
                ),
            }
        }

        match &*stmt.arg {
            Expr::Ident(i) => {
                let name = i.sym.to_string();
                let value = scope
                    .resolve(&name, realm)?
                    .ok_or(Error::reference_error(format!("{name} is not defined")))?;
                let up = update(value, stmt.op);
                
                let ret = if stmt.prefix {
                    up.0.copy()
                } else {
                    up.1
                };
                
                scope.update_or_define(name, up.0);
                
                
                
                Ok(ret)
            }
            Expr::Member(m) => {
                let value = Self::run_member(realm, m, scope)?;

                let up = update(value, stmt.op);
                
                let ret = if stmt.prefix {
                    up.0.copy()
                } else {
                    up.1
                };

                Self::assign_member(realm, m, up.0, scope);
                Ok(ret)
            }

            e => {
                let value = Self::run_expr(realm, e, stmt.span, scope)?;
                //TODO: this isn't correct
                match stmt.op {
                    UpdateOp::PlusPlus => {
                        let value = value - Value::Number(if stmt.prefix { -1.0 } else { 0.0 });
                        Ok(value)
                    }
                    UpdateOp::MinusMinus => {
                        let value = value - Value::Number(if stmt.prefix { 1.0 } else { 0.0 });
                        Ok(value)
                    }
                }
            }
        }
    }
}
