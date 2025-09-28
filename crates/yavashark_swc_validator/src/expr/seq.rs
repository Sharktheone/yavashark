use crate::Validator;
use swc_ecma_ast::SeqExpr;

impl<'a> Validator<'a> {
    pub fn validate_seq_expr(&mut self, seq: &'a SeqExpr) -> Result<(), String> {
        for expr in &seq.exprs {
            self.validate_expr(expr)?;
        }

        Ok(())
    }
}
