use crate::context::Context;
use crate::scope::Scope;
use crate::Error;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::ArrayLit;
use yavashark_value::Obj;
use crate::object::array::Array;

impl Context {
    pub fn run_array(&mut self, stmt: &ArrayLit, scope: &mut Scope) -> RuntimeResult {
        let mut arr = Array::new(self)?;
        
        for elem in &stmt.elems {
            if let Some(elem) = elem {
                if let Some(spread) = elem.spread {
                    let iter = self.run_expr(&elem.expr, spread, scope)?;
                    
                    let mut iter = iter.iter(self)?;
                    for value in iter {
                        arr.push(value?);
                    }
                    
                    
                    
                } else {
                    let value = self.run_expr(&elem.expr, stmt.span, scope)?;
                    arr.push(value);
                }
                
            } else {
                arr.push(Value::Undefined);
            }
        }
        
        Ok(arr.into_value())
    }
}
