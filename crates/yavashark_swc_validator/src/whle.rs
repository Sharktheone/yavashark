use crate::Validator;
use swc_ecma_ast::WhileStmt;

impl Validator {
    pub fn validate_while(with: &WhileStmt) -> Result<(), String> {
        Self::validate_expr(&with.test)?;
        Self::validate_statement(&with.body)
    }
}
