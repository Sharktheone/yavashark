use crate::Validator;
use swc_ecma_ast::LabeledStmt;

impl<'a> Validator<'a> {
    pub fn validate_labeled(&mut self, labeled: &'a LabeledStmt) -> Result<(), String> {
        self.validate_ident(&labeled.label)?;
        self.validate_statement(&labeled.body)
    }
}
