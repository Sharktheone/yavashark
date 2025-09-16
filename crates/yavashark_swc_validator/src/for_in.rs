use swc_ecma_ast::ForInStmt;
use crate::Validator;

impl Validator {
    pub fn validate_for_in(_for_in: &ForInStmt) -> Result<(), String> {
        Ok(())
    }
}
