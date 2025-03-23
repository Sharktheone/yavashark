use yavashark_env::Res;
use crate::data::{Data, OutputData};
use crate::VM;


pub fn b_xor(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.xor(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn b_or(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.or(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn b_and(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.and(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn l_shift(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.shr(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn r_shift(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.shl(&right, vm.get_realm())?;

    output.set(result, vm)
}

pub fn zero_fill_r_shift(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.ushr(&right, vm.get_realm())?;

    output.set(result, vm)
}