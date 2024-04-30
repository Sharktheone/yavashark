use swc_ecma_ast::{Expr, UpdateExpr, UpdateOp};

use yavashark_value::error::Error;

use crate::{ControlFlow, RuntimeResult};
use crate::context::Context;
use crate::scope::Scope;
use crate::Value;

impl Context {
    pub fn run_update(&mut self, stmt: &UpdateExpr, scope: &mut Scope) -> RuntimeResult {
        
        fn update(value: &mut Value, op: UpdateOp) -> Value {
            match op {
                UpdateOp::PlusPlus => *value += Value::Number(1.0),
                UpdateOp::MinusMinus => *value -= Value::Number(1.0),
            }
            value.copy()
        }
        
        match &*stmt.arg {
            Expr::Ident(i) => {
                let value = scope.get_ident_mut(i).ok_or(ControlFlow::error_reference(format!("{i} is not defined")))?;
                let mut scope = value.scope.try_borrow_mut().map_err(|_| Error::new("Cannot borrow mutably".to_string()))?;
                let value = scope.get_ref_mut(&value.name).ok_or(Error::new("Cannot find variable".to_string()))?;
                Ok(update(value, stmt.op))
            }
            Expr::Member(m) => {
                let value = scope.get_member_mut(self, stmt.span, m)?;
                let mut obj = value.obj.try_borrow_mut().map_err(|_| Error::new("Cannot borrow mutably".to_string()))?;
                let obj = obj.get_property_mut(&value.name).ok_or(Error::new("Cannot find object".to_string()))?;
                Ok(update(obj, stmt.op))
            }
            
            e => {
                let value = self.run_expr(e, stmt.span, scope)?;
                match stmt.op {
                    UpdateOp::PlusPlus => {
                        let value = value + Value::Number(1.0);
                        Ok(value)
                    }
                    UpdateOp::MinusMinus => {
                        let value = value - Value::Number(1.0);
                        Ok(value)
                    }
                }
            }
        }

    }
}
