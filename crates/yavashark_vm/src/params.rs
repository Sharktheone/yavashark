use std::ops::Deref;
use yavashark_bytecode::BytecodeFunctionParams;
use yavashark_env::scope::Scope;
use yavashark_env::{Error, Realm, Res, Value};
use crate::{BorrowedVM, VmState, VM};

#[derive(Debug, PartialEq, Clone, Default)]
pub struct VMParams(BytecodeFunctionParams);

impl From<BytecodeFunctionParams> for VMParams {
    fn from(value: BytecodeFunctionParams) -> Self {
        Self(value)
    }
}

impl Deref for VMParams {
    type Target = BytecodeFunctionParams;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl VMParams {
    pub fn execute(&self, args: &[Value], mut scope: Scope, realm: &mut Realm) -> Res {
        let mut prev_i = 0;
        
        for (i, param) in self.0.defs.iter().enumerate() {
            
            let Some(instructions) = self.instructions.get(prev_i..*param as usize) else {
                return Err(Error::new("Invalid parameter instructions"));
            };
            
            prev_i = *param as usize;
            
            let mut vm = BorrowedVM::with_scope(instructions, &self.ds, realm, scope);
            let arg = args.get(i).cloned().unwrap_or(Value::Undefined);
            
            vm.set_acc(arg);
            
            vm.run()?;
            
            scope = vm.current_scope;
        }
        
        Ok(())
    }
}