use crate::VM;
use yavashark_bytecode::{Reg, VarName};
use yavashark_env::Res;

pub fn bitwise_xor(lhs: VarName, rhs: VarName, vm: &mut impl VM) -> Res {
    let lhs = vm.get_variable(lhs)?;
    let rhs = vm.get_variable(rhs)?;
    
    let res = lhs.xor(&rhs, vm.get_realm())?;

    vm.set_acc(res);

    Ok(())
}

pub fn bitwise_xor_acc(reg: Reg, vm: &mut impl VM) -> Res {
    let rhs = vm.get_register(reg)?;
    let lhs = vm.acc();
    
    let res = lhs.xor(&rhs, vm.get_realm())?;

    vm.set_acc(res);

    Ok(())
}

pub fn bitwise_xor_reg(rhs: Reg, lhs: Reg, vm: &mut impl VM) -> Res {
    let rhs = vm.get_register(rhs)?;
    let lhs = vm.get_register(lhs)?;
    
    let res = lhs.xor(&rhs, vm.get_realm())?;

    vm.set_acc(res);

    Ok(())
}
