use crate::Compiler;
use crate::compiler::statement::expr::MoveOptimization;
use swc_ecma_ast::Ident;
use yavashark_bytecode::data::{OutputData, VarName};
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_ident(
        &mut self,
        ident: &Ident,
        out: Option<impl OutputData>,
    ) -> Option<MoveOptimization> {
        out.map(|out| {
            let name = ident.sym.as_str();
            let var = self.alloc_var(name);

            MoveOptimization::new(
                var,
                vec![Instruction::load_var(var, out)],
            )
        })
    }

    pub fn get_ident(&mut self, ident: &Ident) -> VarName {
        let name = ident.sym.as_str();
        self.alloc_var(name)
    }
}
