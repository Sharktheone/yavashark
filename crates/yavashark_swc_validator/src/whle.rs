use crate::Validator;
use swc_ecma_ast::WhileStmt;
use crate::utils::single_stmt_contains_decl;

impl Validator {
    pub fn validate_while(whle: &WhileStmt) -> Result<(), String> {
        Self::validate_expr(&whle.test)?;

        if single_stmt_contains_decl(&whle.body) {
            return Err("Lexical declaration cannot appear in a single-statement context".to_string());
        }

        Self::validate_statement(&whle.body)
    }
}
