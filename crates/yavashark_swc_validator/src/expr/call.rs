use crate::Validator;
use swc_ecma_ast::{CallExpr, Callee};

impl<'a> Validator<'a> {
    pub fn validate_call_expr(&mut self, call: &'a CallExpr) -> Result<(), String> {
        match &call.callee {
            Callee::Expr(expr) => self.validate_expr(expr)?,
            Callee::Super(_) => {}
            Callee::Import(_) => {}
        }

        for arg in &call.args {
            self.validate_expr(&arg.expr)?;
        }

        Ok(())
    }
}
