use swc_ecma_ast::TaggedTpl;
use crate::Validator;

impl Validator {
    pub fn validate_tagged_tpl_expr(tagged_tpl: &TaggedTpl) -> Result<(), String> {
        Self::validate_expr(&tagged_tpl.tag)?;
        Self::validate_tpl_expr(&tagged_tpl.tpl)
    }
}
