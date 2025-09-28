use crate::Validator;
use swc_ecma_ast::NewExpr;

impl<'a> Validator<'a> {
    pub fn validate_new_expr(&mut self, new: &'a NewExpr) -> Result<(), String> {
        self.validate_expr(&new.callee)?;

        if let Some(args) = &new.args {
            for arg in args {
                self.validate_expr(&arg.expr)?;
            }
        }

        Ok(())
    }
}
