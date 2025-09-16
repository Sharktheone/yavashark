use swc_ecma_ast::WhileStmt;
use crate::Validator;

impl Validator {
    pub fn validate_while(_with: &WhileStmt) -> Result<(), String> {
        Ok(())
    }
}
