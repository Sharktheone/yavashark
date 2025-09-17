use swc_ecma_ast::{CallExpr, Callee};
use crate::Validator;

impl Validator {
    pub fn validate_call_expr(call: &CallExpr) -> Result<(), String> {
        match &call.callee {
            Callee::Expr(expr) => Self::validate_expr(expr)?,
            Callee::Super(_) => {},
            Callee::Import(_) => {},
        }
        
        for arg in &call.args {
            Self::validate_expr(&arg.expr)?;
        }
        
        Ok(())
        
    }
}
