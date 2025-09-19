use swc_ecma_ast::Lit;
use crate::Validator;

impl Validator {
    pub fn validate_lit(_lit: &Lit) -> Result<(), String> {
        Ok(())
    }
}
