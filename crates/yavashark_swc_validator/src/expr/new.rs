use crate::Validator;
use swc_ecma_ast::NewExpr;

impl Validator {
    pub fn validate_new_expr(new: &NewExpr) -> Result<(), String> {
        Self::validate_expr(&new.callee)?;

        if let Some(args) = &new.args {
            for arg in args {
                Self::validate_expr(&arg.expr)?;
            }
        }

        Ok(())
    }
}
