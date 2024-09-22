use crate::VM;
use yavashark_bytecode::{ConstIdx, Reg};
use yavashark_env::{Error, Res};

pub fn push_const(const_idx: ConstIdx, vm: &mut VM) -> Res {
    let value = vm.get_constant(const_idx)?;
    vm.push(value);

    Ok(())
}

pub fn push_reg(reg: Reg, vm: &mut VM) -> Res {
    let value = vm.get_register(reg)?;
    vm.push(value);

    Ok(())
}

pub fn push_acc(vm: &mut VM) {
    let value = vm.acc();
    vm.push(value);
}

pub fn pop(vm: &mut VM) {
    vm.pop();
}

pub fn pop_n(n: u32, vm: &mut VM) {
    for _ in 0..n {
        vm.pop();
    }
}

pub fn pop_to_reg(reg: Reg, vm: &mut VM) -> Res {
    let value = vm.pop().ok_or(Error::new("Stack is empty"))?;
    vm.set_register(reg, value)?;

    Ok(())
}

pub fn pop_to_acc(vm: &mut VM) -> Res {
    let value = vm.pop().ok_or(Error::new("Stack is empty"))?;
    vm.set_acc(value);

    Ok(())
}

pub fn stack_to_reg(reg: Reg, vm: &mut VM) -> Res {
    let value = vm.pop().ok_or(Error::new("Stack is empty"))?;
    vm.set_register(reg, value)?;

    Ok(())
}

pub fn stack_to_acc(vm: &mut VM) -> Res {
    let value = vm.pop().ok_or(Error::new("Stack is empty"))?;
    vm.set_acc(value);

    Ok(())
}

pub fn stack_idx_to_reg(reg: Reg, idx: u32, vm: &mut VM) -> Res {
    let value = vm.get_stack(idx).ok_or(Error::new("Stack index out of bounds"))?;
    vm.set_register(reg, value)?;

    Ok(())
}

pub fn stack_idx_to_acc(idx: u32, vm: &mut VM) -> Res {
    let value = vm.get_stack(idx).ok_or(Error::new("Stack index out of bounds"))?;
    vm.set_acc(value);

    Ok(())
}

pub fn reg_to_acc(reg: Reg, vm: &mut VM) -> Res {
    let value = vm.get_register(reg)?;
    vm.set_acc(value);

    Ok(())
}

pub fn acc_to_reg(reg: Reg, vm: &mut VM) -> Res {
    let value = vm.acc();
    vm.set_register(reg, value)?;

    Ok(())
}
