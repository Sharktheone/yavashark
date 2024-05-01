use std::thread::spawn;
use crate::context::Context;
use crate::scope::Scope;
use crate::{ControlFlow, Object, RuntimeResult};
use crate::Value;
use swc_ecma_ast::NewExpr;
use yavashark_value::error::Error;

impl Context {
    pub fn run_new(&mut self, stmt: &NewExpr, scope: &mut Scope) -> RuntimeResult {
        let callee = self.run_expr(&stmt.callee, stmt.span, scope)?;

        if let Value::Object(obj) = callee {
            let mut obj = obj
                .try_borrow_mut()
                .map_err(|_| Error::new("Cannot borrow object".to_string()))?;

            if let Some(f) = &mut obj.call {
                let mut call_args = Vec::with_capacity(0);

                if let Some(args) = &stmt.args {
                    call_args.reserve(args.len());
                    
                    for arg in args {
                        call_args.push(self.run_expr(&arg.expr, arg.spread.unwrap_or(stmt.span), scope)?);
                        if arg.spread.is_some() {
                            todo!("spread")
                        }
                    } 
                }

                let this: Value = Object::new().into();
                

                let _ = f.call(self, call_args, this.copy())?;
                
                Ok(this) //This is always an object, so it will also be updated when we copy it
            } else {
                Err(ControlFlow::error_type(format!("{:?} ia not a constructor", stmt.callee)))
            }
        } else {
            Err(ControlFlow::error_type(format!("{:?} ia not a constructor", stmt.callee)))
        }

    }
}
