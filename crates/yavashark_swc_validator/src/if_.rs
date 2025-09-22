use crate::Validator;
use swc_ecma_ast::IfStmt;

impl Validator {
    pub fn validate_if(if_: &IfStmt) -> Result<(), String> {
        Self::validate_expr(&if_.test)?;
        Self::validate_statement(&if_.cons)?;
        if let Some(alt) = &if_.alt {
            Self::validate_statement(alt)?;
        }

        Ok(())
    }
}
