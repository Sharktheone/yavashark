use swc_ecma_ast::BlockStmt;
use crate::Validator;

impl Validator {
    pub fn validate_block(block: &BlockStmt) -> Result<(), String> {
        Self::validate_statements(&block.stmts)
    }
}