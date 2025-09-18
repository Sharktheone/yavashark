use swc_ecma_ast::PropName;
use crate::Validator;

impl Validator {
    pub fn validate_prop_name(prop_name: &PropName) -> Result<(), String> {
        if let PropName::Computed(computed) = prop_name {
            Self::validate_expr(&computed.expr)?;
        }
        
        Ok(())
    }
}
