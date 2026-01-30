use crate::{Compiler, Res};
use std::rc::Rc;
use swc_ecma_ast::{BlockStmt, Function};
use yavashark_bytecode::data::{ConstIdx, DataSection};
use yavashark_bytecode::{BytecodeFunctionCode, ConstValue, FunctionBlueprint};

impl Compiler {
    pub fn create_function(&mut self, f: &Function, name: Option<String>) -> Res<ConstIdx> {
        let name = name
            .or(self.current_fn_name.take());

        let bp = FunctionBlueprint {
            name,
            params: f.params.clone(),
            is_async: f.is_async,
            is_generator: f.is_generator,
            code: Rc::new(Self::create_bytecode(f)?),
        };

        Ok(self.alloc_const(ConstValue::Function(bp)))
    }

    pub fn create_bytecode(f: &Function) -> Res<BytecodeFunctionCode> {
        if let Some(body) = &f.body {
            return Self::create_bytecode_from_block(body);
        }

        Ok(BytecodeFunctionCode::default())
    }

    pub fn create_bytecode_from_block(b: &BlockStmt) -> Res<BytecodeFunctionCode> {
        let mut this = Self::new();

        this.compile_block(b)?;

        let ds = DataSection::new(this.variables, this.labeled, this.literals, this.control);

        Ok(BytecodeFunctionCode {
            instructions: this.instructions,
            ds,
        })
    }
}
