use crate::Validator;
use swc_ecma_ast::SeqExpr;

impl Validator {
    pub fn validate_seq_expr(seq: &SeqExpr) -> Result<(), String> {
        for expr in &seq.exprs {
            Self::validate_expr(expr)?;
        }

        Ok(())
    }
}
