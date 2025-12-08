use crate::Validator;
use crate::utils::check_async_generator_fn_decl;
use swc_ecma_ast::WithStmt;

impl<'a> Validator<'a> {
    pub fn validate_with(&mut self, with: &'a WithStmt) -> Result<(), String> {
        if self.in_strict_mode() {
            return Err("'with' statement is not allowed in strict mode".to_string());
        }

        self.validate_expr(&with.obj)?;

        check_async_generator_fn_decl(&with.body, "a 'with' statement")?;

        self.validate_statement(&with.body)
    }
}
