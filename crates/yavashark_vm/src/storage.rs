use yavashark_bytecode::VarName;
use yavashark_env::Value;
use crate::VM;

impl VM {
    
    
    pub fn get_variable(&self, name: VarName) -> Value {
        todo!()
    }
    
    pub fn get_register(&self, reg: usize) -> Value {
        todo!()
    }
    
    pub fn set_variable(&mut self, name: VarName, value: Value) {
        todo!()
    }
    
    pub fn set_register(&mut self, reg: usize, value: Value) {
        todo!()
    }
    
    pub fn push(&mut self, value: Value) {
        todo!()
    }
    
    pub fn pop(&mut self) -> Value {
        todo!()
    }
    
    pub fn acc(&self) -> Value {
        todo!()
    }
    
    pub fn set_acc(&mut self, value: Value) {
        todo!()
    }
}