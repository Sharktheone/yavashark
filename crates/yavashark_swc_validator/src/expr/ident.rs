use swc_ecma_ast::Ident;
use crate::Validator;

impl Validator {
    pub fn validate_ident(_ident: &Ident) -> Result<(), String> {
        Ok(())
    }
}
