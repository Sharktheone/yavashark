use crate::Validator;
use crate::utils::check_async_generator_fn_decl;
use swc_ecma_ast::{Decl, LabeledStmt, Stmt};

impl<'a> Validator<'a> {
    pub fn validate_labeled(&mut self, labeled: &'a LabeledStmt) -> Result<(), String> {
        self.validate_ident(&labeled.label)?;

        check_async_generator_fn_decl(&labeled.body, "a labeled statement")?;
        
        if self.in_strict_mode() {
            if let Stmt::Decl(Decl::Fn(_)) = &*labeled.body {
                return Err(
                    "In strict mode, function declarations are not allowed in labelled statements"
                        .to_string(),
                );
            }
        }

        self.validate_statement(&labeled.body)
    }
}
