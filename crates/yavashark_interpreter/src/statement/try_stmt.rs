use swc_ecma_ast::{ObjectPatProp, Pat, PropName, TryStmt};

use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;

impl Context {
    pub fn run_try(&mut self, stmt: &TryStmt, scope: &mut Scope) -> RuntimeResult {
        let res = catch(self, stmt, scope);

        if let Some(finalizer) = &stmt.finalizer {
            let _ = self.run_block(finalizer, scope)?;
        }

        res
    }
}

fn catch(ctx: &mut Context, stmt: &TryStmt, scope: &mut Scope) -> RuntimeResult {
    let try_block = ctx.run_block(&stmt.block, scope);

    if let Err(e) = try_block {
        let err = e.get_error()?;
        if let Some(catch) = &stmt.handler {
            let scope = &mut Scope::with_parent(scope)?;
            if let Some(param) = &catch.param {
                //TODO: Error must be an object, then replace it with self.run_pat
                match param {
                    Pat::Ident(ident) => {
                        scope.declare_var(ident.sym.to_string(), format!("{err:?}").into());
                        //TODO impl Obj for Error
                    }
                    Pat::Object(obj) => {
                        for prop in &obj.props {
                            match prop {
                                ObjectPatProp::Assign(assign) => {
                                    match assign.key.sym.to_string().as_str() {
                                        "message" => {
                                            scope.declare_var(
                                                "message".to_string(),
                                                Value::String(err.message()),
                                            );
                                        }
                                        "stack" => {
                                            scope.declare_var(
                                                "stack".to_string(),
                                                format!("{:?}", err.stack()).into(),
                                            ); //TODO impl Obj for StackTrace
                                        }
                                        "name" => {
                                            scope
                                                .declare_var("name".to_string(), err.name().into());
                                        }
                                        "fileName" => {
                                            scope.declare_var(
                                                "fileName".to_string(),
                                                err.file_name().into(),
                                            );
                                        }
                                        "lineNumber" => {
                                            scope.declare_var(
                                                "lineNumber".to_string(),
                                                err.line_number().into(),
                                            );
                                        }
                                        "columnNumber" => {
                                            scope.declare_var(
                                                "columnNumber".to_string(),
                                                err.column_number().into(),
                                            );
                                        }
                                        (name) => {
                                            let value = if let Some(v) = assign.value.as_ref() {
                                                ctx.run_expr(v, assign.span, scope)?
                                            } else {
                                                Value::Undefined
                                            };
                                            scope.declare_var(name.to_string(), value);
                                        }
                                    }
                                }
                                ObjectPatProp::KeyValue(kv) => {
                                    let key = match &kv.key {
                                        PropName::Ident(ident) => ident.sym.to_string(),
                                        _ => {
                                            todo!()
                                        }
                                    };

                                    let name = match *kv.value {
                                        Pat::Ident(ref ident) => ident.sym.to_string(),
                                        _ => {
                                            todo!()
                                        }
                                    };

                                    match key.as_str() {
                                        "message" => {
                                            scope.declare_var(name, err.message().into());
                                        }
                                        "stack" => {
                                            scope.declare_var(
                                                name,
                                                format!("{:?}", err.stack()).into(),
                                            );
                                        }
                                        "name" => {
                                            scope.declare_var(name, err.name().into());
                                        }
                                        "fileName" => {
                                            scope.declare_var(name, err.file_name().into());
                                        }
                                        "lineNumber" => {
                                            scope.declare_var(name, err.line_number().into());
                                        }
                                        "columnNumber" => {
                                            scope.declare_var(name, err.column_number().into());
                                        }
                                        (_) => {
                                            scope.declare_var(name, Value::Undefined);
                                        }
                                    }
                                }
                                ObjectPatProp::Rest(_) => {
                                    todo!()
                                }
                            }
                        }
                    }
                    _ => {}
                }
            }

            ctx.run_block(&catch.body, scope)
        } else {
            Err(err.into())
        }
    } else {
        try_block
    }
}

#[cfg(test)]
mod tests {
    use crate::{test_eval, Value};

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
            try {
                throw new Error("error message");
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
