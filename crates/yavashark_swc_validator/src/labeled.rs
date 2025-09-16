use swc_ecma_ast::LabeledStmt;
use crate::Validator;

impl Validator {
    pub fn validate_labeled(_labeled: &LabeledStmt) -> Result<(), String> {
        Ok(())
    }
}
