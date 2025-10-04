use swc_ecma_ast::NewExpr;

use crate::Interpreter;
use yavashark_env::scope::Scope;
use yavashark_env::utils::ValueIterator;
use yavashark_env::{ControlFlow, Object, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_new(realm: &mut Realm, stmt: &NewExpr, scope: &mut Scope) -> RuntimeResult {
        let callee = Self::run_expr(realm, &stmt.callee, stmt.span, scope)?;

        let Value::Object(constructor) = callee.copy() else {
            return Err(ControlFlow::error_type(format!(
                "{:?} is not a constructor",
                stmt.callee
            )));
        };

        let mut call_args = Vec::new();

        if let Some(args) = &stmt.args {
            call_args.reserve(args.len());

            for arg in args {
                let res = Self::run_expr(realm, &arg.expr, arg.spread.unwrap_or(stmt.span), scope)?;
                if arg.spread.is_some() {
                    let iter = ValueIterator::new(&res, realm)?;

                    while let Some(value) = iter.next(realm)? {
                        call_args.push(value);
                    }
                } else {
                    call_args.push(res);
                }
            }
        }

        Ok(constructor.construct(call_args, realm)?.into())
    }
}
