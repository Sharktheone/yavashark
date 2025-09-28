use crate::Validator;
use swc_ecma_ast::ReturnStmt;

impl<'a> Validator<'a> {
    pub fn validate_return(&mut self, ret: &'a ReturnStmt) -> Result<(), String> {
        if let Some(arg) = &ret.arg {
            self.validate_expr(arg)?;
        }

        Ok(())
    }
}
