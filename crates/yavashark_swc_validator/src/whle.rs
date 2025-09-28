use crate::Validator;
use swc_ecma_ast::WhileStmt;
use crate::utils::single_stmt_contains_decl;

impl<'a> Validator<'a> {
    pub fn validate_while(&mut self, whle: &'a WhileStmt) -> Result<(), String> {
        self.validate_expr(&whle.test)?;

        if single_stmt_contains_decl(&whle.body) {
            return Err("Lexical declaration cannot appear in a single-statement context".to_string());
        }

        self.validate_statement(&whle.body)
    }
}
