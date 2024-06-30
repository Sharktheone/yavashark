use crate::Interpreter;
use swc_ecma_ast::ArrayLit;
use yavashark_env::array::Array;
use yavashark_env::scope::Scope;
use yavashark_env::value::Obj;
use yavashark_env::{Context, RuntimeResult, Value};

impl Interpreter {
    pub fn run_array(ctx: &mut Context, stmt: &ArrayLit, scope: &mut Scope) -> RuntimeResult {
        let mut arr = Array::from_ctx(ctx);

        for elem in &stmt.elems {
            if let Some(elem) = elem {
                if let Some(spread) = elem.spread {
                    let iter = Self::run_expr(ctx, &elem.expr, spread, scope)?;

                    let mut iter = iter.iter(ctx)?;
                    for value in iter {
                        arr.push(value?);
                    }
                } else {
                    let value = Self::run_expr(ctx, &elem.expr, stmt.span, scope)?;
                    arr.push(value);
                }
            } else {
                arr.push(Value::Undefined);
            }
        }

        Ok(arr.into_value())
    }
}
