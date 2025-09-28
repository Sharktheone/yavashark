use crate::Validator;
use swc_ecma_ast::TaggedTpl;

impl<'a> Validator<'a> {
    pub fn validate_tagged_tpl_expr(&mut self, tagged_tpl: &'a TaggedTpl) -> Result<(), String> {
        self.validate_expr(&tagged_tpl.tag)?;
        self.validate_tpl_expr(&tagged_tpl.tpl)
    }
}
