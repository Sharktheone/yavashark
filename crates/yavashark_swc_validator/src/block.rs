use swc_ecma_ast::BlockStmt;
use crate::Validator;

impl Validator {
    pub fn validate_block(_block: &BlockStmt) -> Result<(), String> {
        Ok(())
    }
}