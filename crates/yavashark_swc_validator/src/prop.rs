use crate::Validator;
use swc_ecma_ast::PropName;

impl<'a> Validator<'a> {
    pub fn validate_prop_name(&mut self, prop_name: &'a PropName) -> Result<(), String> {
        match prop_name {
            PropName::Ident(ident) => self.validate_ident_name(ident)?,
            PropName::Computed(computed) => self.validate_expr(&computed.expr)?,
            _ => {}
        }

        Ok(())
    }
}
