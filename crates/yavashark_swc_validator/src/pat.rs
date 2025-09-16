use swc_ecma_ast::Pat;
use crate::Validator;

impl Validator {
    pub fn validate_pat(_pat: &Pat) -> Result<(), String> {
        Ok(())
    }
}
