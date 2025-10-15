use crate::Validator;
use crate::pat::collect_bound_names;
use crate::utils::block_has_use_strict;
use swc_ecma_ast::{FnExpr, Function, Ident, Param, Pat};

impl<'a> Validator<'a> {
    pub fn validate_function_expr(&mut self, function: &'a FnExpr) -> Result<(), String> {
        if let Some(ident) = &function.ident {
            self.validate_ident(ident)?;
        }

        self.validate_function(&function.function, function.ident.as_ref(), false, false)
    }

    pub fn validate_function(
        &mut self,
        function: &'a Function,
        name: Option<&'a Ident>,
        allow_super_property: bool,
        allow_super_call: bool,
    ) -> Result<(), String> {
        let ctx = self.enter_function_context(function.is_async, function.is_generator);

        self.set_super_property_allowed(allow_super_property);
        self.set_super_call_allowed(allow_super_call);

        if let Some(body) = &function.body {
            if block_has_use_strict(body) {
                self.set_current_function_strict();
            }
        }

        let relaxed_await = if !function.is_async && self.await_restriction_depth > 0 {
            Some(self.enter_relaxed_await_scope())
        } else {
            None
        };

        let check_duplicate_params = !self.in_strict_mode()
            && !function.is_async
            && !function.is_generator
            && is_simple_parameter_list(&function.params);

        let mut seen_params = if check_duplicate_params {
            None
        } else {
            Some(Vec::new())
        };

        for param in &function.params {
            if let Err(e) = self.validate_pat_internal(&param.pat, &mut seen_params) {
                if let Some(relax) = relaxed_await {
                    relax.exit(self);
                }
                ctx.exit(self);

                return Err(e);
            }
        }

        let mut param_names = if let Some(params) = seen_params {
            params
        } else {
            let mut names = Vec::new();
            for param in &function.params {
                collect_bound_names(&param.pat, &mut names);
            }
            names
        };

        for name in param_names.drain(..) {
            self.register_param_name(name);
        }

        if let Some(body) = &function.body {
            if let Err(e) = self.validate_block_with_shadow(body, false) {
                if let Some(relax) = relaxed_await {
                    relax.exit(self);
                }
                ctx.exit(self);

                return Err(e);
            }
        }

        if let Some(name) = name {
            if self.in_strict_mode() && matches!(name.sym.as_ref(), "eval" | "arguments") {
                if let Some(relax) = relaxed_await {
                    relax.exit(self);
                }
                ctx.exit(self);
                return Err(format!(
                    "Identifier '{}' is not allowed in strict mode",
                    name.sym
                ));
            }
        }

        if let Some(relax) = relaxed_await {
            relax.exit(self);
        }

        ctx.exit(self);

        Ok(())
    }
}

fn is_simple_parameter_list(params: &[Param]) -> bool {
    params
        .iter()
        .all(|param| matches!(&param.pat, Pat::Ident(_)))
}
