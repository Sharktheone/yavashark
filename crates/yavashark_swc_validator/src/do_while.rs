use crate::Validator;
use crate::utils::single_stmt_contains_decl;
use swc_ecma_ast::DoWhileStmt;

impl<'a> Validator<'a> {
    pub fn validate_do_while(&mut self, do_while: &'a DoWhileStmt) -> Result<(), String> {
        self.validate_expr(&do_while.test)?;

        if single_stmt_contains_decl(&do_while.body) {
            return Err("Lexical declaration cannot appear in a single-statement context".to_string());
        }

        self.validate_statement(&do_while.body)
    }
}
