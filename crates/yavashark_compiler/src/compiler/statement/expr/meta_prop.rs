use crate::{Compiler, Res};
use swc_ecma_ast::{MetaPropExpr, MetaPropKind};
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;

impl Compiler {
    pub fn compile_meta_prop(&mut self, expr: &MetaPropExpr, out: Option<impl OutputData>) -> Res {
        let Some(out) = out else { return Ok(()) };

        match &expr.kind {
            MetaPropKind::NewTarget => {
                self.instructions.push(Instruction::get_new_target(out));
            }
            MetaPropKind::ImportMeta => {
                self.instructions.push(Instruction::get_import_meta(out));
            }
        }

        Ok(())
    }
}
