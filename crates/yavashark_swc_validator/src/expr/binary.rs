use crate::Validator;
use swc_ecma_ast::BinExpr;

impl<'a> Validator<'a> {
    pub fn validate_binary_expr(&mut self, bin: &'a BinExpr) -> Result<(), String> {
        self.validate_expr(&bin.left)?;
        self.validate_expr(&bin.right)
    }
}
