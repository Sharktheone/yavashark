mod var;

use crate::Validator;
use swc_ecma_ast::Decl;

impl<'a> Validator<'a> {
    pub fn validate_decl(&mut self, decl: &'a Decl) -> Result<(), String> {
        match decl {
            Decl::Class(class_decl) => self.validate_class(&class_decl.class),
            Decl::Fn(fn_decl) => self.validate_function(&fn_decl.function),
            Decl::Var(var_decl) => self.validate_var_decl(var_decl),
            _ => Err("Unsupported declaration type".to_string()),
        }
    }
}
