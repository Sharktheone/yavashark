use crate::Validator;
use swc_ecma_ast::Tpl;

impl<'a> Validator<'a> {
    pub fn validate_tpl_expr(&mut self, tpl: &'a Tpl) -> Result<(), String> {
        for expr in &tpl.exprs {
            self.validate_expr(expr)?;
        }
        Ok(())
    }
}
