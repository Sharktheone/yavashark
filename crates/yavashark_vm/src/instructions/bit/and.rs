use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn bitwise_and(lhs: VarName, rhs: VarName, vm: &mut impl VM) -> Res {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;

    
    let res = rhs.and(&lhs, vm.get_realm())?;
    
    vm.set_acc(res);

    Ok(())
}

pub fn bitwise_and_acc(reg: Reg, vm: &mut impl VM) -> Res {
    let rhs = vm.get_register(reg)?;
    let lhs = vm.acc();

    
    let res = rhs.and(&lhs, vm.get_realm())?;
    
    vm.set_acc(res);

    Ok(())
}

pub fn bitwise_and_reg(rhs: Reg, lhs: Reg, vm: &mut impl VM) -> Res {
    let rhs = vm.get_register(rhs)?;
    let lhs = vm.get_register(lhs)?;

    
    let res = rhs.and(&lhs, vm.get_realm())?;
    
    vm.set_acc(res);

    Ok(())
}
