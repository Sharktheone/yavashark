use swc_ecma_ast::Ident;
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;
use crate::Compiler;
use crate::compiler::statement::expr::MoveOptimization;

impl Compiler {
    pub fn compile_ident(&mut self, ident: &Ident, out: Option<impl OutputData>) -> Option<MoveOptimization> {
        if let Some(out) = out {
            let name = ident.sym.as_str();
            let var = self.alloc_var(name);
            
            Some(MoveOptimization::new(var, vec![Instruction::load_var(var, out)]))
        } else {
            None
        }
    }
}