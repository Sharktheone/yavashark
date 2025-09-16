use swc_ecma_ast::ThrowStmt;
use crate::Validator;

impl Validator {
    pub fn validate_throw(_throw: &ThrowStmt) -> Result<(), String> {
        Ok(())
    }
}
