use crate::Validator;
use swc_ecma_ast::IfStmt;

impl<'a> Validator<'a> {
    pub fn validate_if(&mut self, if_: &'a IfStmt) -> Result<(), String> {
        self.validate_expr(&if_.test)?;
        self.validate_statement(&if_.cons)?;
        if let Some(alt) = &if_.alt {
            self.validate_statement(alt)?;
        }

        Ok(())
    }
}
