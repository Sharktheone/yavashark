use swc_ecma_ast::DoWhileStmt;
use crate::Validator;

impl Validator {
    pub fn validate_do_while(_decl: &DoWhileStmt) -> Result<(), String> {
        Ok(())
    }
}
