use crate::Validator;
use crate::utils::{block_has_use_strict, statements_have_use_strict};
use swc_ecma_ast::{ArrowExpr, ArrowFunctionBody, BlockStmtOrExpr};

impl<'a> Validator<'a> {
    pub fn validate_arrow_expr(&mut self, arrow: &'a ArrowExpr) -> Result<(), String> {
        let scope = self.enter_function_context(arrow.is_async, false);

        if let ArrowFunctionBody::FunctionBody(block) = &*arrow.body
            && statements_have_use_strict(&block.stmts)
        {
            self.set_current_function_strict();
        }

        let mut seen_params = Some(Vec::new());

        for param in &arrow.params {
            if let Err(e) = self.validate_pat_internal(param, &mut seen_params) {
                scope.exit(self);

                return Err(e);
            }
        }

        if let Some(params) = seen_params {
            for name in params {
                self.register_param_name(name);
            }
        }

        let res = match &*arrow.body {
            ArrowFunctionBody::FunctionBody(block) => {
                self.validate_block_with_shadow(&block.stmts, false)
            }
            ArrowFunctionBody::Expr(expr) => self.validate_expr(expr),
        };

        scope.exit(self);

        res
    }
}
