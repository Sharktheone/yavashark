use crate::Interpreter;
use swc_common::Spanned;
use swc_ecma_ast::AwaitExpr;
use yavashark_env::builtins::{Promise, PromiseState};
use yavashark_env::error_obj::ErrorObj;
use yavashark_env::scope::Scope;
use yavashark_env::{ControlFlow, Error, Realm, RuntimeResult, Value};

impl Interpreter {
    pub fn run_await(realm: &mut Realm, stmt: &AwaitExpr, scope: &mut Scope) -> RuntimeResult {
        let val = Self::run_expr(realm, &stmt.arg, stmt.arg.span(), scope)?;

        if let Value::Object(obj) = &val {
            if let Some(p) = obj.downcast::<Promise>() {
                let state = p.state.get();

                let inner = p.inner.borrow();
                let value = inner.value.clone().unwrap_or(Value::Undefined);

                return match state {
                    PromiseState::Fulfilled => Ok(value),
                    PromiseState::Rejected => Err(ControlFlow::Error(Error::throw(value))),
                    PromiseState::Pending => Err(ControlFlow::Error(Error::new(
                        "TODO: await not yet implemented in interpreter, use bytecode!",
                    ))),
                };
            }
        }

        Ok(val)
    }
}
