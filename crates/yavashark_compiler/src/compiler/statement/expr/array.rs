use super::MoveOptimization;
use crate::{Compiler, Res};
use swc_ecma_ast::{ArrayLit, ExprOrSpread, PropOrSpread};
use yavashark_bytecode::data::OutputData;
use yavashark_bytecode::instructions::Instruction;
use yavashark_bytecode::{ArrayLiteralBlueprint, ConstValue};

impl Compiler {
    pub fn compile_array(
        &mut self,
        expr: &ArrayLit,
        out: Option<impl OutputData>,
    ) -> Res<Option<MoveOptimization>> {
        let Some(out) = out else {
            return Ok(None);
        };

        let mut properties = Vec::with_capacity(expr.elems.len());
        let mut dealloc = Vec::new();

        for elem in &expr.elems {
            match elem {
                Some(expr) => {
                    let storage = self.alloc_reg_or_stack();
                    dealloc.push(storage);

                    self.compile_expr_data_certain(&expr.expr, storage);

                    properties.push(Some(storage.into()));
                }
                None => {
                    properties.push(None);
                }
            }
        }
        let ob = self.alloc_const(ConstValue::Array(ArrayLiteralBlueprint { properties }));

        let m = MoveOptimization::new(ob, vec![Instruction::move_(ob, out)]);

        for dealloc in dealloc {
            self.dealloc(dealloc);
        }

        Ok(Some(m))
    }
}
