use swc_ecma_ast::ThisExpr;
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;
use crate::Compiler;

impl Compiler {
    pub fn compile_this(&mut self, _this: &ThisExpr, out: Option<impl OutputData>) {
        if let Some(out) = out {
            self.instructions.push(Instruction::this(out));
            
        }
    }
}