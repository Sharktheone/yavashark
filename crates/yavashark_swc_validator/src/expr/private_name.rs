use swc_ecma_ast::PrivateName;
use crate::Validator;

impl Validator {
    pub fn validate_private_name_expr(_private_name: &PrivateName) -> Result<(), String> {
        Ok(())
    }
}
