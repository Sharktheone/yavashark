use std::rc::Rc;
use swc_ecma_ast::Function;
use yavashark_bytecode::{BytecodeFunctionCode, ConstValue, FunctionBlueprint};
use yavashark_bytecode::data::{ConstIdx, DataSection};
use crate::{Compiler, Res};

impl Compiler {
    pub fn create_function(&mut self, f: &Function, name: Option<String>) -> Res<ConstIdx> {
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
        let mut this = Self::new();
        
        if let Some(body) = &f.body {
            this.compile_block(body)?;
        }
        
        let ds = DataSection::new(this.variables, this.labeled, this.literals);
        
        Ok(BytecodeFunctionCode {
            instructions: this.instructions,
            ds,
        })
    }
}