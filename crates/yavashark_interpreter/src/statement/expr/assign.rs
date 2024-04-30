use crate::context::Context;
use crate::scope::{AssignValue, Scope};
use crate::{ControlFlow, RuntimeResult};
use crate::Value;
use swc_ecma_ast::{AssignExpr, AssignTarget, SimpleAssignTarget};
use yavashark_value::error::Error;

impl Context {
    pub fn run_assign(&mut self, stmt: &AssignExpr, scope: &mut Scope) -> RuntimeResult {
        let value = self.run_expr(&stmt.right, stmt.span, scope)?;
        
        match &stmt.left { 
            AssignTarget::Simple(t) => {
                let target = scope.get_assign_target(self, t)?;
                
                match target {
                    AssignValue::MutValue(v) => {
                        let mut scope = v.scope.try_borrow_mut().map_err(|_| Error::new("Cannot borrow mutably".to_string()))?;
                        let var = scope.get_ref_mut(&v.name).ok_or(Error::new("Cannot find variable".to_string()))?;
                        
                        *var = value;
                        
                        Ok(Value::Undefined)
                    }
                    AssignValue::MutObject(o) => {
                        let mut obj = o.obj.try_borrow_mut().map_err(|_| Error::new("Cannot borrow mutably".to_string()))?;
                        let obj = obj.get_property_mut(&o.name).ok_or(Error::new("Cannot find object".to_string()))?;
                        
                        *obj = value;
                        
                        Ok(Value::Undefined)
                    }
                    AssignValue::OptChainNone => Ok(Value::Undefined),
                }
            }
            AssignTarget::Pat(_) => {
                if !matches!(value, Value::Object(_)) {
                    return Err(ControlFlow::error("Invalid left-hand side in assignment".to_string()));
                }
                todo!("Pattern assignment")
            }
        }
    }
}
