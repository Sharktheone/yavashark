use crate::{instructions, VM};
use yavashark_bytecode::Instruction;
use yavashark_env::ControlResult;

trait Execute {
    fn execute(&self, vm: &mut VM);
}

impl Execute for Instruction {
    fn execute(&self, vm: &mut VM) -> ControlResult {
        match self {
            Instruction::Add(lhs, rhs) => instructions::add(*lhs, *rhs, vm),
            Instruction::AddAccReg(reg) => instructions::add_acc_reg(*reg, vm),
            Instruction::AddReg(lhs, rhs) => instructions::add_reg(*lhs, *rhs, vm),

            Instruction::Sub(lhs, rhs) => instructions::sub(*lhs, *rhs, vm),
            Instruction::SubAccReg(reg) => instructions::sub_acc_reg(*reg, vm),
            Instruction::SubReg(lhs, rhs) => instructions::sub_reg(*lhs, *rhs, vm),

            Instruction::Div(lhs, rhs) => instructions::div(*lhs, *rhs, vm),
            Instruction::DivAccReg(reg) => instructions::div_acc_reg(*reg, vm),
            Instruction::DivReg(lhs, rhs) => instructions::div_reg(*lhs, *rhs, vm),

            Instruction::Mul(lhs, rhs) => instructions::mul(*lhs, *rhs, vm),
            Instruction::MulAccReg(reg) => instructions::mul_acc_reg(*reg, vm),
            Instruction::MulReg(lhs, rhs) => instructions::mul_reg(*lhs, *rhs, vm),

            Instruction::Mod(lhs, rhs) => instructions::modulo(*lhs, *rhs, vm),
            Instruction::ModAccReg(reg) => instructions::mod_acc_reg(*reg, vm),
            Instruction::ModReg(lhs, rhs) => instructions::mod_reg(*lhs, *rhs, vm),

            Instruction::LNot(var_name) => instructions::logical_not(*var_name, vm),
            Instruction::LNotAcc => instructions::logical_not_acc(vm),

            Instruction::LOr(lhs, rhs) => instructions::logical_or(*lhs, *rhs, vm),
            Instruction::LOrAcc(reg) => instructions::logical_or_acc(*reg, vm),

            Instruction::LAnd(lhs, rhs) => instructions::logical_and(*lhs, *rhs, vm),
            Instruction::LAndAcc(reg) => instructions::logical_and_acc(*reg, vm),

            Instruction::BitXor(lhs, rhs) => instructions::bitwise_xor(*lhs, *rhs, vm),
            Instruction::BitXorAcc(reg) => instructions::bitwise_xor_acc(*reg, vm),
            Instruction::BitXorReg(lhs, rhs) => instructions::bitwise_xor_reg(*lhs, *rhs, vm),

            Instruction::BitOr(lhs, rhs) => instructions::bitwise_or(*lhs, *rhs, vm),
            Instruction::BitOrAcc(reg) => instructions::bitwise_or_acc(*reg, vm),
            Instruction::BitOrReg(lhs, rhs) => instructions::bitwise_or_reg(*lhs, *rhs, vm),

            Instruction::BitAnd(lhs, rhs) => instructions::bitwise_and(*lhs, *rhs, vm),
            Instruction::BitAndAcc(reg) => instructions::bitwise_and_acc(*reg, vm),
            Instruction::BitAndReg(lhs, rhs) => instructions::bitwise_and_reg(*lhs, *rhs, vm),

            Instruction::Eq(lhs, rhs) => instructions::eq(*lhs, *rhs, vm),
            Instruction::EqAcc(reg) => instructions::eq_acc(*reg, vm),
            Instruction::EqReg(lhs, rhs) => instructions::eq_reg(*lhs, *rhs, vm),

            Instruction::NotEq(lhs, rhs) => instructions::not_eq(*lhs, *rhs, vm),
            Instruction::NotEqAcc(reg) => instructions::not_eq_acc(*reg, vm),
            Instruction::NotEqReg(lhs, rhs) => instructions::not_eq_reg(*lhs, *rhs, vm),

            Instruction::EqEq(lhs, rhs) => instructions::eq_eq(*lhs, *rhs, vm),
            Instruction::EqEqAcc(reg) => instructions::eq_eq_acc(*reg, vm),
            Instruction::EqEqReg(lhs, rhs) => instructions::eq_eq_reg(*lhs, *rhs, vm),

            Instruction::NotEqEq(lhs, rhs) => instructions::not_eq_eq(*lhs, *rhs, vm),
            Instruction::NotEqEqAcc(reg) => instructions::not_eq_eq_acc(*reg, vm),
            Instruction::NotEqEqReg(lhs, rhs) => instructions::not_eq_eq_reg(*lhs, *rhs, vm),

            Instruction::Lt(lhs, rhs) => instructions::lt(*lhs, *rhs, vm),
            Instruction::LtAcc(reg) => instructions::lt_acc(*reg, vm),
            Instruction::LtReg(lhs, rhs) => instructions::lt_reg(*lhs, *rhs, vm),

            Instruction::LtEq(lhs, rhs) => instructions::lt_eq(*lhs, *rhs, vm),
            Instruction::LtEqAcc(reg) => instructions::lt_eq_acc(*reg, vm),
            Instruction::LtEqReg(lhs, rhs) => instructions::lt_eq_reg(*lhs, *rhs, vm),

            Instruction::Gt(lhs, rhs) => instructions::gt(*lhs, *rhs, vm),
            Instruction::GtAcc(reg) => instructions::gt_acc(*reg, vm),
            Instruction::GtReg(lhs, rhs) => instructions::gt_reg(*lhs, *rhs, vm),

            Instruction::GtEq(lhs, rhs) => instructions::gt_eq(*lhs, *rhs, vm),
            Instruction::GtEqAcc(reg) => instructions::gt_eq_acc(*reg, vm),
            Instruction::GtEqReg(lhs, rhs) => instructions::gt_eq_reg(*lhs, *rhs, vm),

            Instruction::LShift(lhs, rhs) => instructions::lshift(*lhs, *rhs, vm),
            Instruction::LShiftAcc(reg) => instructions::lshift_acc(*reg, vm),
            Instruction::LShiftReg(lhs, rhs) => instructions::lshift_reg(*lhs, *rhs, vm),

            Instruction::RShift(lhs, rhs) => instructions::rshift(*lhs, *rhs, vm),
            Instruction::RShiftAcc(reg) => instructions::rshift_acc(*reg, vm),
            Instruction::RShiftReg(lhs, rhs) => instructions::rshift_reg(*lhs, *rhs, vm),

            Instruction::ZeroFillRShift(lhs, rhs) => instructions::zero_fill_rshift(*lhs, *rhs, vm),
            Instruction::ZeroFillRShiftAcc(reg) => instructions::zero_fill_rshift_acc(*reg, vm),
            Instruction::ZeroFillRShiftReg(lhs, rhs) => {
                instructions::zero_fill_rshift_reg(*lhs, *rhs, vm)
            }

            Instruction::In(lhs, rhs) => instructions::in_(*lhs, *rhs, vm)?,
            Instruction::InAcc(reg) => instructions::in_acc(*reg, vm)?,
            Instruction::InReg(lhs, rhs) => instructions::in_reg(*lhs, *rhs, vm)?,

            Instruction::InstanceOf(target, name) => *instructions::instance_of(*target, *name, vm),
            Instruction::InstanceOfAcc(reg) => instructions::instance_of_acc(*reg, vm),
            Instruction::InstanceOfReg(target, reg) => {
                instructions::instance_of_reg(*target, *reg, vm)
            }

            Instruction::Exp(target, name) => instructions::exp(*target, *name, vm),
            Instruction::ExpAcc(reg) => instructions::exp_acc(*reg, vm),
            Instruction::ExpReg(target, reg) => instructions::exp_reg(*target, *reg, vm),

            Instruction::NullishCoalescing(target, name) => {
                instructions::nullish_coalescing(*target, *name, vm)
            }
            Instruction::NullishCoalescingAcc(reg) => {
                instructions::nullish_coalescing_acc(*reg, vm)
            }
            Instruction::NullishCoalescingReg(target, reg) => {
                instructions::nullish_coalescing_reg(*target, *reg, vm)
            }

            Instruction::Dec(name) => instructions::dec(*name, vm),
            Instruction::DecAcc => instructions::dec_acc(vm),
            Instruction::DecReg(reg) => instructions::dec_reg(*reg, vm),

            Instruction::Inc(name) => instructions::inc(*name, vm),
            Instruction::IncAcc => instructions::inc_acc(vm),
            Instruction::IncReg(reg) => instructions::inc_reg(*reg, vm),

            Instruction::PushScope => instructions::push_scope(vm),
            Instruction::PopScope => instructions::pop_scope(vm),

            Instruction::Call(num_args, name) => instructions::call(*num_args, *name, vm)?,
            Instruction::CallReg(num_args, reg) => instructions::call_reg(*num_args, *reg, vm)?,
            Instruction::CallAcc(num_args) => instructions::call_acc(*num_args, vm)?,
            Instruction::CallMember(num_args, name, member) => {
                instructions::call_member(*num_args, *name, *member, vm)?
            }
            Instruction::CallMemberReg(num_args, reg, member) => {
                instructions::call_member_reg(*num_args, *reg, *member, vm)?
            }
            Instruction::CallMemberAcc(num_args, member) => {
                instructions::call_member_acc(*num_args, *member, vm)?
            }

            Instruction::Jmp(target) => instructions::jmp(*target, vm),
            Instruction::JmpIf(name, target) => instructions::jmp_if(*target, *name, vm),
            Instruction::JmpIfAcc(target) => instructions::jmp_if_acc(*target, vm),
            Instruction::JmpIfNot(target, name) => instructions::jmp_if_not(*name, *target, vm),
            Instruction::JmpIfNotAcc(target) => instructions::jmp_if_not_acc(*target, vm),
            Instruction::JmpNull(target, name) => instructions::jmp_null(*name, *target, vm),
            Instruction::JmpNullAcc(target) => instructions::jmp_null_acc(*target, vm),
            Instruction::JmpUndef(target, name) => instructions::jmp_undef(*name, *target, vm),
            Instruction::JmpUndefAcc(target) => instructions::jmp_undef_acc(*target, vm),
            Instruction::JmpNullUndef(name, addr) => instructions::jmp_null_undef(*addr, *name, vm),
            Instruction::JmpNullUndefAcc(addr) => instructions::jmp_null_undef_acc(*addr, vm),

            Instruction::JmpRel(target) => instructions::jmp_rel(*target, vm),
            Instruction::JmpIfRel(target, name) => instructions::jmp_if_rel(*name, *target, vm),
            Instruction::JmpIfAccRel(target) => instructions::jmp_if_acc_rel(*target, vm),
            Instruction::JmpIfNotRel(target, name) => {
                instructions::jmp_if_not_rel(*name, *target, vm)
            }
            Instruction::JmpIfNotAccRel(target) => instructions::jmp_if_not_acc_rel(*target, vm),
            Instruction::JmpNullRel(target, name) => instructions::jmp_null_rel(*name, *target, vm),
            Instruction::JmpNullAccRel(target) => instructions::jmp_null_acc_rel(*target, vm),
            Instruction::JmpUndefRel(target, name) => {
                instructions::jmp_undef_rel(*name, *target, vm)
            }
            Instruction::JmpUndefAccRel(target) => instructions::jmp_undef_acc_rel(*target, vm),
            Instruction::JmpNullUndefRel(name, addr) => {
                instructions::jmp_null_undef_rel(*addr, *name, vm)
            }
            Instruction::JmpNullUndefAccRel(addr) => {
                instructions::jmp_null_undef_acc_rel(*addr, vm)
            }

            // Instruction::Str(name, const_idx) => instructions::str(name, const_idx, vm),
            // Instruction::StrAcc(const_idx) => instructions::str_acc(const_idx, vm),
            // Instruction::StrReg(name, const_idx) => instructions::str_reg(name, const_idx, vm),
            Instruction::Lda(name, const_idx) => instructions::lda(*name, *const_idx, vm),
            Instruction::LdaAcc(const_idx) => instructions::lda_acc(*const_idx, vm),
            Instruction::LdaReg(name, const_idx) => instructions::lda_reg(*name, *const_idx, vm),

            Instruction::LoadMemberAcc(member) => instructions::load_member_acc(*member, vm)?,
            Instruction::LoadMemberReg(target, member) => {
                instructions::load_member_reg(*target, *member, vm)?
            }
            Instruction::LoadRegMember(target, member) => {
                instructions::load_member(*target, *member, vm)?
            }
            // Instruction::LoadRegMemberAcc(reg, member) => {
            //     instructions::load_reg_member_acc(reg, member, vm)
            // }
            // Instruction::LoadAccMember(member, reg) => {
            //     instructions::load_acc_member(member, reg, vm)
            // }
            // Instruction::LoadAccMemberAcc(member) => instructions::load_acc_member_acc(member, vm),
            Instruction::LoadEnv(name) => instructions::load_env(name, vm),
            Instruction::LoadEnvReg(name, reg) => instructions::load_env_reg(name, reg, vm),

            Instruction::For => instructions::for_(vm),

            Instruction::TypeOf(name) => instructions::type_of(*name, vm),
            Instruction::TypeOfAcc => instructions::type_of_acc(vm),

            Instruction::PushConst(idx) => instructions::push_const(*idx, vm),
            Instruction::PushReg(reg) => instructions::push_reg(*reg, vm),
            Instruction::PushAcc => instructions::push_acc(vm),
            Instruction::Pop => instructions::pop(vm),
            Instruction::PopN(b) => instructions::pop_n(*b, vm),
            Instruction::PopToReg(reg) => instructions::pop_to_reg(*reg, vm),
            Instruction::PopToAcc => instructions::pop_to_acc(vm),
            Instruction::StackToReg(reg) => instructions::stack_to_reg(reg, vm),
            Instruction::StackToAcc => instructions::stack_to_acc(vm),
            Instruction::StackIdxToReg(reg, idx) => instructions::stack_idx_to_reg(reg, idx, vm),
            Instruction::StackIdxToAcc(idx) => instructions::stack_idx_to_acc(idx, vm),
            Instruction::RegToAcc(reg) => instructions::reg_to_acc(reg, vm),
            Instruction::AccToReg(reg) => instructions::acc_to_reg(reg, vm),

            Instruction::Return => instructions::return_()?,
            Instruction::ReturnAcc => instructions::return_acc(vm)?,
            Instruction::ReturnReg(reg) => instructions::return_reg(*reg, vm)?,
            Instruction::ReturnVar(name) => instructions::return_var(*name, vm)?,

            Instruction::ThrowAcc => instructions::throw_acc(vm)?,
            Instruction::ThrowReg(reg) => instructions::throw_reg(*reg, vm)?,
            Instruction::Throw(name) => instructions::throw(*name, vm)?,

            Instruction::ThisAcc => instructions::this_acc(vm),
            Instruction::ThisReg(reg) => instructions::this_reg(*reg, vm),
        }

        Ok(())
    }
}
