use swc_ecma_ast::{ObjectPatProp, Pat, PropName, TryStmt};

use crate::{RuntimeResult, Value};
use crate::context::Context;
use crate::scope::Scope;

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
            let scope = &mut Scope::with_parent(scope);
            if let Some(param) = &catch.param {
                match param {
                    Pat::Ident(ident) => {
                        scope.declare_var(ident.sym.to_string(), err.into());
                    }
                    Pat::Object(obj) => {
                        for prop in &obj.props {
                            match prop {
                                ObjectPatProp::Assign(assign) => {
                                    match assign.key.sym.to_string().as_str() {
                                        "message" => {
                                            scope.declare_var("message".to_string(), err.message().into());
                                        }
                                        "stack" => {
                                            scope.declare_var("stack".to_string(), err.stack().into());
                                        }
                                        "name" => {
                                            scope.declare_var("name".to_string(), err.name().into());
                                        }
                                        "fileName" => {
                                            scope.declare_var("fileName".to_string(), err.file_name().into());
                                        }
                                        "lineNumber" => {
                                            scope.declare_var("lineNumber".to_string(), err.line_number().into());
                                        }
                                        "columnNumber" => {
                                            scope.declare_var("columnNumber".to_string(), err.column_number().into());
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
                                        PropName::Ident(ident) => {
                                            ident.sym.to_string()
                                        }
                                        _ => {todo!()}
                                    };
                                    
                                    let name = match *kv.value {
                                        Pat::Ident(ref ident) => ident.sym.to_string(),
                                        _ => {todo!()}
                                    };
                                    
                                    match key.as_str() {
                                        "message" => {
                                            scope.declare_var(name, err.message().into());
                                        }
                                        "stack" => {
                                            scope.declare_var(name, err.stack().into());
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
                                _ => {todo!()}
                            }
                        }
                    }
                    _ => {}
                }
            }

            Ok(Value::Undefined)
        } else {
            Ok(Value::Undefined)
        }
    } else {
        try_block
    }
}
