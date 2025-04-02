use crate::data::{Data, OutputData};
use yavashark_env::Res;
use crate::VM;

pub fn add_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;

    let ret = val.add(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn sub_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;

    let ret = val.sub(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn mul_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;

    let ret = val.mul(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn div_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;

    let ret = val.div(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn rem_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;

    let ret = val.rem(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn l_shift_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;
    
    let ret = val.shl(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn r_shift_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;
    
    let ret = val.shr(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn zero_fill_r_shift_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;
    
    let ret = val.ushr(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn b_and_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;
    
    let ret = val.ushr(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn b_or_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;
    
    let ret = val.or(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn b_xor_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;
    
    let ret = val.xor(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn exp_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;
    
    let ret = val.exp(&out, vm.get_realm())?;

    output.set(ret, vm)
}

pub fn and_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;

    let ret = val.log_and(out);

    output.set(ret, vm)
}

pub fn or_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let val = val.get(vm)?;
    let out = output.get(vm)?;

    let ret = val.log_or(out);

    output.set(ret, vm)
}

pub fn nullish_assign(val: impl Data, output: impl OutputData, vm: &mut impl VM) -> Res {
    let out = output.get(vm)?;
    
    if out.is_nullish() {
        let val = val.get(vm)?;
        output.set(val, vm)
    } else {
        Ok(())
    }
}

