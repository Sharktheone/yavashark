use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;
use crate::VM;

pub fn load_env(name: VarName, vm: &mut VM) -> Res {
    let value = vm.get_variable(name)?;
    
    vm.set_acc(value);
    
    Ok(())
}

pub fn load_env_reg(name: VarName, reg: Reg, vm: &mut VM) -> Res {
    let value = vm.get_variable(name)?;
    
    vm.set_register(reg, value)?;
    
    Ok(())
}
