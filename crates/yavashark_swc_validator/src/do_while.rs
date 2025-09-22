use crate::Validator;
use swc_ecma_ast::DoWhileStmt;

impl Validator {
    pub fn validate_do_while(do_while: &DoWhileStmt) -> Result<(), String> {
        Self::validate_expr(&do_while.test)?;
        Self::validate_statement(&do_while.body)
    }
}
