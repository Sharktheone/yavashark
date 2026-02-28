use crate::Validator;
use swc_ecma_ast::{Callee, Expr, NewExpr};

impl<'a> Validator<'a> {
    pub fn validate_new_expr(&mut self, new: &'a NewExpr) -> Result<(), String> {
        if let Expr::Call(call) = &*new.callee
            && matches!(call.callee, Callee::Import(_))
        {
            return Err("Cannot use 'new' with dynamic import".to_string());
        }

        self.validate_expr(&new.callee)?;

        if let Some(args) = &new.args {
            for arg in args {
                self.validate_expr(&arg.expr)?;
            }
        }

        Ok(())
    }
}
