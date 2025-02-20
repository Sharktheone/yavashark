use crate::VM;
use yavashark_bytecode::{ConstIdx, Reg, VarName};
use yavashark_env::Res;

pub fn lda(name: VarName, const_idx: ConstIdx, vm: &mut impl VM) -> Res {
    let value = vm.get_constant(const_idx)?;

    vm.set_variable(name, value)?;

    Ok(())
}

pub fn lda_acc(const_idx: ConstIdx, vm: &mut impl VM) -> Res {
    let value = vm.get_constant(const_idx)?;

    vm.set_acc(value);

    Ok(())
}

pub fn lda_reg(reg: Reg, const_idx: ConstIdx, vm: &mut impl VM) -> Res {
    let value = vm.get_constant(const_idx)?;

    vm.set_register(reg, value)?;

    Ok(())
}
