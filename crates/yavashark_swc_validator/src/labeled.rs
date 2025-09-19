use crate::Validator;
use swc_ecma_ast::LabeledStmt;

impl Validator {
    pub fn validate_labeled(labeled: &LabeledStmt) -> Result<(), String> {
        Self::validate_statement(&labeled.body)
    }
}
