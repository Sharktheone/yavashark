use super::MoveOptimization;
use crate::{Compiler, Res};
use swc_ecma_ast::FnExpr;
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_fn(
        &mut self,
        expr: &FnExpr,
        out: Option<impl OutputData>,
    ) -> Res<Option<MoveOptimization>> {
        let Some(out)  = out else {
            return Ok(None);
        };
        
        let f = self.create_function(&expr.function, None)?;
        
        Ok(Some(MoveOptimization::new(f, vec![Instruction::move_(f, out)])))
    }
}
