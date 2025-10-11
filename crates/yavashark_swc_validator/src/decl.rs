mod var;

use crate::Validator;
use swc_ecma_ast::Decl;

impl<'a> Validator<'a> {
    pub fn validate_decl(&mut self, decl: &'a Decl) -> Result<(), String> {
        match decl {
            Decl::Class(class_decl) => {
                self.ensure_not_function_param(&class_decl.ident.sym)?;
                self.validate_class(&class_decl.class)
            }
            Decl::Fn(fn_decl) => {
                if fn_decl.ident.sym.as_ref() == "await" && self.is_await_restricted() {
                } else {
                    self.validate_ident(&fn_decl.ident)?;
                }
                self.validate_function(&fn_decl.function, Some(&fn_decl.ident), false, false)
            }
            Decl::Var(var_decl) => self.validate_var_decl(var_decl),
            _ => Err("Unsupported declaration type".to_string()),
        }
    }
}
