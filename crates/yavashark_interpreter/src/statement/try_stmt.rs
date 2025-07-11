use crate::Interpreter;
use std::iter;
use swc_ecma_ast::{ObjectPatProp, Pat, PropName, TryStmt};
use yavashark_env::error::ErrorObj;
use yavashark_env::scope::Scope;
use yavashark_env::{Realm, RuntimeResult, Value};
use yavashark_value::ErrorKind;

impl Interpreter {
    pub fn run_try(realm: &mut Realm, stmt: &TryStmt, scope: &mut Scope) -> RuntimeResult {
        let res = catch(realm, stmt, scope);

        if let Some(finalizer) = &stmt.finalizer {
            let _ = Self::run_block(realm, finalizer, scope)?;
        }

        res
    }
}

fn catch(realm: &mut Realm, stmt: &TryStmt, scope: &mut Scope) -> RuntimeResult {
    let try_block = Interpreter::run_block(realm, &stmt.block, scope);

    if let Err(e) = try_block {
        let err = e.get_error()?;
        if let Some(catch) = &stmt.handler {
            let scope = &mut Scope::with_parent(scope)?;
            if let Some(param) = &catch.param {
                let err = ErrorObj::error_to_value(err, realm);

                Interpreter::run_pat(
                    realm,
                    param,
                    scope,
                    &mut iter::once(err),
                    &mut |scope, name, value| {
                        scope.declare_var(name, value)?;
                        Ok(())
                    },
                )?;
            }

            Interpreter::run_block(realm, &catch.body, scope)
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
    use yavashark_value::ErrorKind;

    #[test]
    fn try_stmt() {
        test_eval!(
            r#"
            try {
                throw new Error("error message");
            } catch ({message}) {
                mock.send()
                message
            }
            "#,
            1,
            Vec::<Vec<Value>>::new(),
            Value::String("error message".into())
        );
    }

    #[test]
    fn try_catch_with_error_thrown() {
        test_eval!(
            r#"
            
            ret = undefined;
            try {
                throw new Error("error message");
            } catch (e) {
                mock.send()
                ret = e.message
            }
            ret
            "#,
            1,
            Vec::<Vec<Value>>::new(),
            Value::String("error message".into())
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
            Value::String("no error".into())
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
            Value::String("finalizer executed".into())
        );
    }

    #[test]
    fn try_catch_with_no_error_thrown_and_finalizer() {
        test_eval!(
            r#"

            let ret = undefined;

            try {
                ret = "no error"
            } catch (e) {
                e.message
            } finally {
                ret = "finalizer executed"
            }


            ret
            "#,
            0,
            Vec::<Vec<Value>>::new(),
            Value::String("finalizer executed".into())
        );
    }

    #[test]
    fn try_catch_ret_finalizer() {
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
            Value::String("no error".into())
        );
    }

    #[test]
    fn try_catch_with_error_thrown_and_no_catch_block() {
        let (result, value) = test_eval!(
            r#"

            try {
                throw new Error("error message");
            } finally {
                mock.send()
            }

            "#
        );

        assert!(result.is_err());

        let err = result.unwrap_err();

        assert_eq!(err.kind, ErrorKind::Error(Some("error message".into())));

        let state = value.borrow();

        assert_eq!(state.send_called, 1);
    }
}
