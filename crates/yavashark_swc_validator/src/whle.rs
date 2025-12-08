use crate::Validator;
use crate::utils::{single_stmt_contains_decl, check_async_generator_fn_decl};
use swc_ecma_ast::WhileStmt;

impl<'a> Validator<'a> {
    pub fn validate_while(&mut self, whle: &'a WhileStmt) -> Result<(), String> {
        self.validate_expr(&whle.test)?;

        if single_stmt_contains_decl(&whle.body) {
            return Err(
                "Lexical declaration cannot appear in a single-statement context".to_string(),
            );
        }

        check_async_generator_fn_decl(&whle.body, "a 'while' statement")?;

        self.validate_statement(&whle.body)
    }
}
