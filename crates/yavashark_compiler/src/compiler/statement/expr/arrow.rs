use std::rc::Rc;
use anyhow::anyhow;
use crate::{Compiler, Res};
use swc_ecma_ast::{ArrowExpr, BlockStmtOrExpr, Param, Pat};
use yavashark_bytecode::{ArrowFunctionBlueprint, BytecodeFunctionCode, ConstValue};
use yavashark_bytecode::data::{DataSection, OutputData};
use yavashark_bytecode::instructions::Instruction;
use crate::compiler::statement::expr::MoveOptimization;

impl Compiler {
    pub fn compile_arrow(&mut self, expr: &ArrowExpr, out: Option<impl OutputData>) -> Res<Option<MoveOptimization>> {
        let Some(out) = out else {
            return Ok(None);
        };
        
        let mut this = Self::new();
        
        match &*expr.body {
            BlockStmtOrExpr::BlockStmt(block) => {
                this.compile_block(block)?;
            }
            BlockStmtOrExpr::Expr(expr) => {
                let out = this.compile_expr_data_acc(&*expr)?;
                
                this.instructions.push(Instruction::return_value(out));
            }
        }

        let ds = DataSection::new(this.variables, this.labeled, this.literals, this.control);

        let code = BytecodeFunctionCode {
            instructions: this.instructions,
            ds,
        };
        
        
        let f = self.alloc_const(ConstValue::ArrowFunction(ArrowFunctionBlueprint {
            params: expr.params.iter().map(|param| Param::from(param.clone())).collect(),
            is_async: expr.is_async,
            is_generator: expr.is_generator,
            code: Rc::new(code),
        }));

        Ok(Some(MoveOptimization::new(
            f,
            vec![Instruction::move_(f, out)],
        )))
    }
}
