use swc_ecma_ast::Tpl;
use crate::Validator;

impl Validator {
    pub fn validate_tpl_expr(tpl: &Tpl) -> Result<(), String> {
        for expr in &tpl.exprs {
            Self::validate_expr(expr)?;
        }
        Ok(())
    }
}
