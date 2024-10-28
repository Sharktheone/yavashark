use swc_ecma_ast::NewExpr;

use yavashark_env::scope::Scope;
use yavashark_env::{Realm, ControlFlow, Object, RuntimeResult, Value};

use crate::Interpreter;

impl Interpreter {
    pub fn run_new(realm: &mut Realm, stmt: &NewExpr, scope: &mut Scope) -> RuntimeResult {
        let callee = Self::run_expr(realm, &stmt.callee, stmt.span, scope)?;

        let Value::Object(constructor) = callee else {
            return Err(ControlFlow::error_type(format!(
                "{:?} is not a constructor1",
                stmt.callee
            )));
        };

        let this = constructor
            .get_constructor_value(realm)
            .ok_or(ControlFlow::error_type(format!(
                "{:?} is not a constructor2",
                stmt.callee
            )))?;

        let f = if constructor.special_constructor()? {
            constructor
        } else {
            let Value::Object(o) = constructor.get_constructor().value else {
                return Err(ControlFlow::error_type(format!(
                    "{:?} is not a constructor3",
                    stmt.callee
                )));
            };

            o
        };

        let mut call_args = Vec::with_capacity(0);

        if let Some(args) = &stmt.args {
            call_args.reserve(args.len());

            for arg in args {
                call_args.push(Self::run_expr(
                    realm,
                    &arg.expr,
                    arg.spread.unwrap_or(stmt.span),
                    scope,
                )?);
                if arg.spread.is_some() {
                    todo!("spread")
                }
            }
        }

        dbg!(f.to_string(realm));

        let _ = f.call(realm, call_args, this.copy())?;

        Ok(this) //This is always an object, so it will also be updated when we copy it
    }
}
