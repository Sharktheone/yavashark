use crate::Validator;
use swc_ecma_ast::{FnExpr, Function};

impl<'a> Validator<'a> {
    pub fn validate_function_expr(&mut self, function: &'a FnExpr) -> Result<(), String> {
        self.validate_function(&function.function)
    }

    pub fn validate_function(&mut self, function: &'a Function) -> Result<(), String> {
        let ctx = self.enter_function_context(function.is_async, function.is_generator);

        let mut seen_params = Some(Vec::new());

        for param in &function.params {
            if let Err(e) = self.validate_pat_internal(&param.pat, &mut seen_params) {
                ctx.exit(self);

                return Err(e);
            }
        }

        if let Some(body) = &function.body {
            if let Err(e) = self.validate_block(body) {
                ctx.exit(self);

                return Err(e);
            }
        }

        ctx.exit(self);

        Ok(())
    }
}
