use crate::context::Context;
use crate::scope::Scope;
use crate::RuntimeResult;
use crate::Value;
use swc_ecma_ast::{UpdateExpr, UpdateOp};
use yavashark_value::error::Error;

impl Context {
    pub fn run_update(&mut self, stmt: &UpdateExpr, scope: &mut Scope) -> RuntimeResult {
        let value = self.run_expr(&stmt.arg, stmt.span, scope)?;

        let value = match stmt.op {
            UpdateOp::PlusPlus => value + Value::Number(1.0),
            UpdateOp::MinusMinus => value + Value::Number(1.0),
        };
        
        todo!("get value mut or something");
        
        Ok(Value::Undefined)
    }
}
