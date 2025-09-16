use swc_ecma_ast::ExprStmt;
use crate::Validator;

impl Validator {
    pub fn validate_expr(_expr: &ExprStmt) -> Result<(), String> {
        Ok(())
    }
}
