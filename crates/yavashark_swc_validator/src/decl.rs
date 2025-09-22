mod var;

use crate::Validator;
use swc_ecma_ast::Decl;

impl Validator {
    pub fn validate_decl(decl: &Decl) -> Result<(), String> {
        match decl {
            Decl::Class(class_decl) => Self::validate_class(&class_decl.class),
            Decl::Fn(fn_decl) => Self::validate_function(&fn_decl.function),
            Decl::Var(var_decl) => Self::validate_var_decl(var_decl),
            _ => Err("Unsupported declaration type".to_string()),
        }
    }
}
