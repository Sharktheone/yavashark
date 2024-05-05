use swc_ecma_ast::Pat;
use crate::context::Context;
use crate::{Res, RuntimeResult, Value};
use crate::scope::Scope;

impl Context {
    pub fn run_pat(&mut self, stmt: &Pat, scope: &mut Scope, value: Value) -> Res {
        match stmt {
            Pat::Ident(id) => {
                scope.declare_var(id.sym.to_string(), value);
            }
            Pat::Array(arr) => {
                for elem in &arr.elems {
                    match elem {
                        Some(elem) => {
                            // let value = self.run_pat(elem, scope, value)?;
                        }
                        None => {}
                    }
                }
            }
            _ => todo!(),
        }
        
        
        Ok(())
    } 
}