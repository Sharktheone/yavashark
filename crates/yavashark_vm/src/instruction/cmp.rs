use yavashark_env::Res;
use crate::data::{Data, OutputData};
use crate::VM;



pub fn eq(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = left.normal_eq(&right).into();

    output.set(result, vm)
}

pub fn ne(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = (!left.normal_eq(&right)).into();

    output.set(result, vm)
}

pub fn strict_eq(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = (left == right).into();

    output.set(result, vm)
}

pub fn strict_ne(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = (left != right).into();

    output.set(result, vm)
}


pub fn lt(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = (left < right).into();

    output.set(result, vm)
}

pub fn lt_eq(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = (left <= right).into();

    output.set(result, vm)
}

pub fn gt(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = (left > right).into();

    output.set(result, vm)
}

pub fn gt_eq(left: impl Data, right: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let left = left.get(vm)?;
    let right = right.get(vm)?;
    let result = (left >= right).into();

    output.set(result, vm)
}
