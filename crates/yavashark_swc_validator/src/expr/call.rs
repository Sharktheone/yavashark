use crate::Validator;
use swc_ecma_ast::{CallExpr, Callee};

impl<'a> Validator<'a> {
    pub fn validate_call_expr(&mut self, call: &'a CallExpr) -> Result<(), String> {
        let is_import_call = matches!(call.callee, Callee::Import(_));

        match &call.callee {
            Callee::Expr(expr) => self.validate_expr(expr)?,
            Callee::Super(_) => {}
            Callee::Import(_) => {}
        }

        if is_import_call {
            if call.args.is_empty() {
                return Err("Dynamic import requires at least one argument".to_string());
            }

            if call.args.len() > 2 {
                return Err("Dynamic import accepts at most two arguments".to_string());
            }
        }

        for arg in &call.args {
            if is_import_call && arg.spread.is_some() {
                return Err(
                    "Dynamic import arguments cannot use the spread operator".to_string(),
                );
            }

            self.validate_expr(&arg.expr)?;
        }

        Ok(())
    }
}
