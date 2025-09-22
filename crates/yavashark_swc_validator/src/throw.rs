use crate::Validator;
use swc_ecma_ast::ThrowStmt;

impl Validator {
    pub fn validate_throw(throw: &ThrowStmt) -> Result<(), String> {
        Self::validate_expr(&throw.arg)
    }
}
