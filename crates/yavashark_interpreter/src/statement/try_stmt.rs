use crate::Interpreter;
use swc_ecma_ast::{ObjectPatProp, Pat, PropName, TryStmt};
use yavashark_env::scope::Scope;
use yavashark_env::{Context, RuntimeResult, Value};
use yavashark_env::error::ErrorObj;

impl Interpreter {
    pub fn run_try(ctx: &mut Context, stmt: &TryStmt, scope: &mut Scope) -> RuntimeResult {
        let res = catch(ctx, stmt, scope);

        if let Some(finalizer) = &stmt.finalizer {
            let _ = Self::run_block(ctx, finalizer, scope)?;
        }

        res
    }
}

fn catch(ctx: &mut Context, stmt: &TryStmt, scope: &mut Scope) -> RuntimeResult {
    let try_block = Interpreter::run_block(ctx, &stmt.block, scope);

    if let Err(e) = try_block {
        let err = e.get_error()?;
        if let Some(catch) = &stmt.handler {
            let scope = &mut Scope::with_parent(scope)?;
            if let Some(param) = &catch.param {
                let err = ErrorObj::new(err, ctx).into();
                
                
                Interpreter::run_pat(ctx, param, scope, err);
            }

            Interpreter::run_block(ctx, &catch.body, scope)
        } else {
            Err(err.into())
        }
    } else {
        try_block
    }
}

#[cfg(test)]
mod tests {
    use yavashark_env::{test_eval, Value};

    #[test]
    fn try_stmt() {
        test_eval!(
            r#"
            try {
                throw new Error("error message");
            } catch ({message}) {
                message
            }
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::String("error message".to_string())
        );
    }

    #[test]
    fn try_catch_with_error_thrown() {
        test_eval!(
            r#"
            try {
                throw new Error("error message");
            } catch (e) {
                e.message
            }
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::String("error message".to_string())
        );
    }

    #[test]
    fn try_catch_without_error_thrown() {
        test_eval!(
            r#"
            try {
                "no error"
            } catch (e) {
                e.message
            }
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::String("no error".to_string())
        );
    }

    #[test]
    fn try_catch_with_error_thrown_and_finalizer() {
        test_eval!(
            r#"
            
            let ret = undefined;
            
            try {
                throw new Error("error message");
            } catch (e) {
                e.message
            } finally {
                ret = "finalizer executed"
            }
            
            
            ret
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::String("finalizer executed".to_string())
        );
    }

    #[test]
    fn try_catch_with_no_error_thrown_and_finalizer() {
        test_eval!(
            r#"
            try {
                "no error"
            } catch (e) {
                e.message
            } finally {
                "finalizer executed"
            }
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::String("finalizer executed".to_string())
        );
    }

    #[test]
    fn try_catch_with_error_thrown_and_no_catch_block() {
        test_eval!(
            r#"
            try {
                throw new Error("error message");
            } finally {
                "finalizer executed"
            }
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::String("finalizer executed".to_string())
        );
    }
}
