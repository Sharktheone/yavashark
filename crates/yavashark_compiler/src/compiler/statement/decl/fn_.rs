use swc_ecma_ast::FnDecl;
use yavashark_bytecode::instructions::Instruction;
use crate::Compiler;

impl Compiler {
    pub fn decl_fn(&mut self, func: &FnDecl) -> crate::Res {
        let f = self.create_function(&func.function, Some(func.ident.sym.to_string()))?;
        
        let name = self.alloc_var(func.ident.sym.as_str());
        
        self.instructions.push(Instruction::move_(f, name));
        
        Ok(())
    }
}
