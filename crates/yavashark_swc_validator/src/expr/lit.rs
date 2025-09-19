use crate::Validator;
use swc_ecma_ast::Lit;

impl Validator {
    pub fn validate_lit(_lit: &Lit) -> Result<(), String> {
        Ok(())
    }
}
