use crate::Validator;
use swc_ecma_ast::WithStmt;

impl Validator {
    pub fn validate_with(with: &WithStmt) -> Result<(), String> {
        Self::validate_expr(&with.obj)?;
        Self::validate_statement(&with.body)
    }
}
