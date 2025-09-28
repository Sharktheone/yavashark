use crate::Validator;
use swc_ecma_ast::TryStmt;

impl<'a> Validator<'a> {
    pub fn validate_try(&mut self, try_: &'a TryStmt) -> Result<(), String> {
        self.validate_block(&try_.block)?;
        if let Some(handler) = &try_.handler {
            if let Some(param) = &handler.param {
                self.validate_pat_dup(param, true)?;
            }

            self.validate_block(&handler.body)?;
        }
        if let Some(finalizer) = &try_.finalizer {
            self.validate_block(finalizer)?;
        }
        Ok(())
    }
}
