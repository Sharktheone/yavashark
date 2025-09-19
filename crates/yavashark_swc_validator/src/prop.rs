use swc_ecma_ast::PropName;
use crate::Validator;

impl Validator {
    pub fn validate_prop_name(prop_name: &PropName) -> Result<(), String> {
        match prop_name {
            PropName::Ident(ident) => Self::validate_ident_name(ident)?,
            PropName::Computed(computed) => Self::validate_expr(&computed.expr)?,
            _ => {}
        }
        
        Ok(())
    }
}
