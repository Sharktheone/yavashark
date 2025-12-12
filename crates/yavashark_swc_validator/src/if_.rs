use crate::Validator;
use crate::utils::{check_async_generator_fn_decl, is_labelled_function};
use swc_ecma_ast::IfStmt;

impl<'a> Validator<'a> {
    pub fn validate_if(&mut self, if_: &'a IfStmt) -> Result<(), String> {
        self.validate_expr(&if_.test)?;

        check_async_generator_fn_decl(&if_.cons, "an 'if' statement")?;

        if is_labelled_function(&if_.cons) {
            return Err(
                "Labelled function declaration is not allowed as the body of an 'if' statement"
                    .to_string(),
            );
        }

        self.validate_statement(&if_.cons)?;

        if let Some(alt) = &if_.alt {
            check_async_generator_fn_decl(alt, "an 'else' statement")?;

            if is_labelled_function(alt) {
                return Err(
                    "Labelled function declaration is not allowed as the body of an 'else' statement"
                        .to_string(),
                );
            }

            self.validate_statement(alt)?;
        }

        Ok(())
    }
}
