use crate::Validator;
use swc_ecma_ast::TryStmt;

impl Validator {
    pub fn validate_try(try_: &TryStmt) -> Result<(), String> {
        Self::validate_block(&try_.block)?;
        if let Some(handler) = &try_.handler {
            if let Some(param) = &handler.param {
                Self::validate_pat(param)?;
            }

            Self::validate_block(&handler.body)?;
        }
        if let Some(finalizer) = &try_.finalizer {
            Self::validate_block(finalizer)?;
        }
        Ok(())
    }
}
