use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn dec(name: VarName, vm: &mut VM) -> Res {
    let val = vm.get_variable(name)?;

    let result = val - 1.into();

    vm.set_acc(result);
    
    Ok(())
}

pub fn dec_acc(vm: &mut VM) -> Res {
    let acc = vm.acc();

    let result = acc - 1.into();

    vm.set_acc(result);
    
    Ok(())
}

pub fn dec_reg(name: Reg, vm: &mut VM) -> Res {
    let val = vm.get_register(name)?;

    let result = val - 1.into();

    vm.set_acc(result);
    
    Ok(())
}
