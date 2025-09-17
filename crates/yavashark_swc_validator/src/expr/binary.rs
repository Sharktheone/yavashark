use swc_ecma_ast::BinExpr;
use crate::Validator;

impl Validator {
    pub fn validate_binary_expr(bin: &BinExpr) -> Result<(), String> {
        Self::validate_expr(&bin.left)?;
        Self::validate_expr(&bin.right)
    }
}
