use crate::Validator;
use swc_ecma_ast::BinExpr;

impl Validator {
    pub fn validate_binary_expr(bin: &BinExpr) -> Result<(), String> {
        Self::validate_expr(&bin.left)?;
        Self::validate_expr(&bin.right)
    }
}
