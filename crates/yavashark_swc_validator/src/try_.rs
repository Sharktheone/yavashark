use swc_ecma_ast::TryStmt;
use crate::Validator;

impl Validator {
    pub fn validate_try(_try: &TryStmt) -> Result<(), String> {
        Ok(())
    }
}
