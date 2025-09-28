use crate::Validator;
use swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr};

impl<'a> Validator<'a> {
    pub fn validate_arrow_expr(&mut self, arrow: &'a ArrowExpr) -> Result<(), String> {
        let scope = self.enter_function_context(arrow.is_async, false);

        for param in &arrow.params {
            if let Err(e) = self.validate_pat_dup(param, true) {
                scope.exit(self);

                return Err(e)
            }
        }

        let res = match &*arrow.body {
            BlockStmtOrExpr::BlockStmt(block) => self.validate_block(block),
            BlockStmtOrExpr::Expr(expr) => self.validate_expr(expr),
        };

        scope.exit(self);

        res
    }
}
