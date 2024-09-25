use crate::{instructions, VM};
use yavashark_bytecode::Instruction;
use yavashark_env::ControlResult;

pub trait Execute {
    fn execute(&self, vm: &mut VM) -> ControlResult;
}

impl Execute for Instruction {
    fn execute(&self, vm: &mut VM) -> ControlResult {
        match self {
            Self::Add(lhs, rhs) => instructions::add(*lhs, *rhs, vm)?,
            Self::AddAccReg(reg) => instructions::add_acc_reg(*reg, vm)?,
            Self::AddReg(lhs, rhs) => instructions::add_reg(*lhs, *rhs, vm)?,

            Self::Sub(lhs, rhs) => instructions::sub(*lhs, *rhs, vm)?,
            Self::SubAccReg(reg) => instructions::sub_acc_reg(*reg, vm)?,
            Self::SubReg(lhs, rhs) => instructions::sub_reg(*lhs, *rhs, vm)?,

            Self::Div(lhs, rhs) => instructions::div(*lhs, *rhs, vm)?,
            Self::DivAccReg(reg) => instructions::div_acc_reg(*reg, vm)?,
            Self::DivReg(lhs, rhs) => instructions::div_reg(*lhs, *rhs, vm)?,

            Self::Mul(lhs, rhs) => instructions::mul(*lhs, *rhs, vm)?,
            Self::MulAccReg(reg) => instructions::mul_acc_reg(*reg, vm)?,
            Self::MulReg(lhs, rhs) => instructions::mul_reg(*lhs, *rhs, vm)?,

            Self::Mod(lhs, rhs) => instructions::modulo(*lhs, *rhs, vm)?,
            Self::ModAccReg(reg) => instructions::mod_acc_reg(*reg, vm)?,
            Self::ModReg(lhs, rhs) => instructions::mod_reg(*lhs, *rhs, vm)?,

            Self::LNot(var_name) => instructions::logical_not(*var_name, vm)?,
            Self::LNotAcc => instructions::logical_not_acc(vm)?,

            Self::LOr(lhs, rhs) => instructions::logical_or(*lhs, *rhs, vm)?,
            Self::LOrAcc(reg) => instructions::logical_or_acc(*reg, vm)?,

            Self::LAnd(lhs, rhs) => instructions::logical_and(*lhs, *rhs, vm)?,
            Self::LAndAcc(reg) => instructions::logical_and_acc(*reg, vm)?,

            Self::BitXor(lhs, rhs) => instructions::bitwise_xor(*lhs, *rhs, vm)?,
            Self::BitXorAcc(reg) => instructions::bitwise_xor_acc(*reg, vm)?,
            Self::BitXorReg(lhs, rhs) => instructions::bitwise_xor_reg(*lhs, *rhs, vm)?,

            Self::BitOr(lhs, rhs) => instructions::bitwise_or(*lhs, *rhs, vm)?,
            Self::BitOrAcc(reg) => instructions::bitwise_or_acc(*reg, vm)?,
            Self::BitOrReg(lhs, rhs) => instructions::bitwise_or_reg(*lhs, *rhs, vm)?,

            Self::BitAnd(lhs, rhs) => instructions::bitwise_and(*lhs, *rhs, vm)?,
            Self::BitAndAcc(reg) => instructions::bitwise_and_acc(*reg, vm)?,
            Self::BitAndReg(lhs, rhs) => instructions::bitwise_and_reg(*lhs, *rhs, vm)?,

            Self::Eq(lhs, rhs) => instructions::eq(*lhs, *rhs, vm)?,
            Self::EqAcc(reg) => instructions::eq_acc(*reg, vm)?,
            Self::EqReg(lhs, rhs) => instructions::eq_reg(*lhs, *rhs, vm)?,

            Self::NotEq(lhs, rhs) => instructions::not_eq(*lhs, *rhs, vm)?,
            Self::NotEqAcc(reg) => instructions::not_eq_acc(*reg, vm)?,
            Self::NotEqReg(lhs, rhs) => instructions::not_eq_reg(*lhs, *rhs, vm)?,

            Self::EqEq(lhs, rhs) => instructions::eq_eq(*lhs, *rhs, vm)?,
            Self::EqEqAcc(reg) => instructions::eq_eq_acc(*reg, vm)?,
            Self::EqEqReg(lhs, rhs) => instructions::eq_eq_reg(*lhs, *rhs, vm)?,

            Self::NotEqEq(lhs, rhs) => instructions::not_eq_eq(*lhs, *rhs, vm)?,
            Self::NotEqEqAcc(reg) => instructions::not_eq_eq_acc(*reg, vm)?,
            Self::NotEqEqReg(lhs, rhs) => instructions::not_eq_eq_reg(*lhs, *rhs, vm)?,

            Self::Lt(lhs, rhs) => instructions::lt(*lhs, *rhs, vm)?,
            Self::LtAcc(reg) => instructions::lt_acc(*reg, vm)?,
            Self::LtReg(lhs, rhs) => instructions::lt_reg(*lhs, *rhs, vm)?,

            Self::LtEq(lhs, rhs) => instructions::lt_eq(*lhs, *rhs, vm)?,
            Self::LtEqAcc(reg) => instructions::lt_eq_acc(*reg, vm)?,
            Self::LtEqReg(lhs, rhs) => instructions::lt_eq_reg(*lhs, *rhs, vm)?,

            Self::Gt(lhs, rhs) => instructions::gt(*lhs, *rhs, vm)?,
            Self::GtAcc(reg) => instructions::gt_acc(*reg, vm)?,
            Self::GtReg(lhs, rhs) => instructions::gt_reg(*lhs, *rhs, vm)?,

            Self::GtEq(lhs, rhs) => instructions::gt_eq(*lhs, *rhs, vm)?,
            Self::GtEqAcc(reg) => instructions::gt_eq_acc(*reg, vm)?,
            Self::GtEqReg(lhs, rhs) => instructions::gt_eq_reg(*lhs, *rhs, vm)?,

            Self::LShift(lhs, rhs) => instructions::lshift(*lhs, *rhs, vm)?,
            Self::LShiftAcc(reg) => instructions::lshift_acc(*reg, vm)?,
            Self::LShiftReg(lhs, rhs) => instructions::lshift_reg(*lhs, *rhs, vm)?,

            Self::RShift(lhs, rhs) => instructions::rshift(*lhs, *rhs, vm)?,
            Self::RShiftAcc(reg) => instructions::rshift_acc(*reg, vm)?,
            Self::RShiftReg(lhs, rhs) => instructions::rshift_reg(*lhs, *rhs, vm)?,

            Self::ZeroFillRShift(lhs, rhs) => {
                instructions::zero_fill_rshift(*lhs, *rhs, vm)?;
            }
            Self::ZeroFillRShiftAcc(reg) => instructions::zero_fill_rshift_acc(*reg, vm)?,
            Self::ZeroFillRShiftReg(lhs, rhs) => {
                instructions::zero_fill_rshift_reg(*lhs, *rhs, vm)?;
            }

            Self::In(lhs, rhs) => instructions::in_(*lhs, *rhs, vm)?,
            Self::InAcc(reg) => instructions::in_acc(*reg, vm)?,
            Self::InReg(lhs, rhs) => instructions::in_reg(*lhs, *rhs, vm)?,

            Self::InstanceOf(target, name) => instructions::instance_of(*target, *name, vm)?,
            Self::InstanceOfAcc(reg) => instructions::instance_of_acc(*reg, vm)?,
            Self::InstanceOfReg(target, reg) => {
                instructions::instance_of_reg(*target, *reg, vm)?;
            }

            Self::Exp(target, name) => instructions::exp(*target, *name, vm)?,
            Self::ExpAcc(reg) => instructions::exp_acc(*reg, vm)?,
            Self::ExpReg(target, reg) => instructions::exp_reg(*target, *reg, vm)?,

            Self::NullishCoalescing(target, name) => {
                instructions::nullish_coalescing(*target, *name, vm)?;
            }
            Self::NullishCoalescingAcc(reg) => {
                instructions::nullish_coalescing_acc(*reg, vm)?;
            }
            Self::NullishCoalescingReg(target, reg) => {
                instructions::nullish_coalescing_reg(*target, *reg, vm)?;
            }

            Self::Dec(name) => instructions::dec(*name, vm)?,
            Self::DecAcc => instructions::dec_acc(vm)?,
            Self::DecReg(reg) => instructions::dec_reg(*reg, vm)?,

            Self::Inc(name) => instructions::inc(*name, vm)?,
            Self::IncAcc => instructions::inc_acc(vm)?,
            Self::IncReg(reg) => instructions::inc_reg(*reg, vm)?,

            Self::PushScope => instructions::push_scope(vm)?,
            Self::PopScope => instructions::pop_scope(vm)?,

            Self::Call(num_args, name) => instructions::call(*num_args, *name, vm)?,
            Self::CallReg(num_args, reg) => instructions::call_reg(*num_args, *reg, vm)?,
            Self::CallAcc(num_args) => instructions::call_acc(*num_args, vm)?,
            Self::CallMember(num_args, name, member) => {
                instructions::call_member(*num_args, *name, *member, vm)?;
            }
            Self::CallMemberReg(num_args, reg, member) => {
                instructions::call_member_reg(*num_args, *reg, *member, vm)?;
            }
            Self::CallMemberAcc(num_args, member) => {
                instructions::call_member_acc(*num_args, *member, vm)?;
            }

            Self::Jmp(target) => instructions::jmp(*target, vm),
            Self::JmpIf(name, target) => instructions::jmp_if(*target, *name, vm)?,
            Self::JmpIfAcc(target) => instructions::jmp_if_acc(*target, vm)?,
            Self::JmpIfNot(target, name) => instructions::jmp_if_not(*name, *target, vm)?,
            Self::JmpIfNotAcc(target) => instructions::jmp_if_not_acc(*target, vm)?,
            Self::JmpNull(target, name) => instructions::jmp_null(*name, *target, vm)?,
            Self::JmpNullAcc(target) => instructions::jmp_null_acc(*target, vm)?,
            Self::JmpUndef(target, name) => instructions::jmp_undef(*name, *target, vm)?,
            Self::JmpUndefAcc(target) => instructions::jmp_undef_acc(*target, vm)?,
            Self::JmpNullUndef(name, addr) => {
                instructions::jmp_null_undef(*addr, *name, vm)?;
            }
            Self::JmpNullUndefAcc(addr) => instructions::jmp_null_undef_acc(*addr, vm)?,

            Self::JmpRel(target) => instructions::jmp_rel(*target, vm),
            Self::JmpIfRel(target, name) => instructions::jmp_if_rel(*name, *target, vm)?,
            Self::JmpIfAccRel(target) => instructions::jmp_if_acc_rel(*target, vm)?,
            Self::JmpIfNotRel(target, name) => {
                instructions::jmp_if_not_rel(*name, *target, vm)?;
            }
            Self::JmpIfNotAccRel(target) => instructions::jmp_if_not_acc_rel(*target, vm)?,
            Self::JmpNullRel(target, name) => {
                instructions::jmp_null_rel(*name, *target, vm)?;
            }
            Self::JmpNullAccRel(target) => instructions::jmp_null_acc_rel(*target, vm)?,
            Self::JmpUndefRel(target, name) => {
                instructions::jmp_undef_rel(*name, *target, vm)?;
            }
            Self::JmpUndefAccRel(target) => instructions::jmp_undef_acc_rel(*target, vm)?,
            Self::JmpNullUndefRel(name, addr) => {
                instructions::jmp_null_undef_rel(*addr, *name, vm)?;
            }
            Self::JmpNullUndefAccRel(addr) => {
                instructions::jmp_null_undef_acc_rel(*addr, vm)?;
            }

            // Instruction::Str(name, const_idx) => instructions::str(name, const_idx, vm)?,
            // Instruction::StrAcc(const_idx) => instructions::str_acc(const_idx, vm)?,
            // Instruction::StrReg(name, const_idx) => instructions::str_reg(name, const_idx, vm)?,
            Self::Lda(name, const_idx) => instructions::lda(*name, *const_idx, vm)?,
            Self::LdaAcc(const_idx) => instructions::lda_acc(*const_idx, vm)?,
            Self::LdaReg(name, const_idx) => instructions::lda_reg(*name, *const_idx, vm)?,

            Self::LoadMemberAcc(member) => instructions::load_member_acc(*member, vm)?,
            Self::LoadMemberReg(target, member) => {
                instructions::load_member_reg(*target, *member, vm)?;
            }
            Self::LoadRegMember(target, member) => {
                instructions::load_member(*target, *member, vm)?;
            }
            // Instruction::LoadRegMemberAcc(reg, member) => {
            //     instructions::load_reg_member_acc(reg, member, vm)
            // }
            // Instruction::LoadAccMember(member, reg) => {
            //     instructions::load_acc_member(member, reg, vm)
            // }
            // Instruction::LoadAccMemberAcc(member) => instructions::load_acc_member_acc(member, vm)?,
            Self::LoadEnv(name) => instructions::load_env(*name, vm)?,
            Self::LoadEnvReg(name, reg) => instructions::load_env_reg(*name, *reg, vm)?,

            Self::TypeOf(name) => instructions::type_of(*name, vm)?,
            Self::TypeOfAcc => instructions::type_of_acc(vm)?,

            Self::PushConst(idx) => instructions::push_const(*idx, vm)?,
            Self::PushReg(reg) => instructions::push_reg(*reg, vm)?,
            Self::PushAcc => instructions::push_acc(vm),
            Self::Pop => instructions::pop(vm),
            Self::PopN(b) => instructions::pop_n(*b, vm),
            Self::PopToReg(reg) => instructions::pop_to_reg(*reg, vm)?,
            Self::PopToAcc => instructions::pop_to_acc(vm)?,
            Self::StackToReg(reg) => instructions::stack_to_reg(*reg, vm)?,
            Self::StackToAcc => instructions::stack_to_acc(vm)?,
            Self::StackIdxToReg(reg, idx) => instructions::stack_idx_to_reg(*reg, *idx, vm)?,
            Self::StackIdxToAcc(idx) => instructions::stack_idx_to_acc(*idx, vm)?,
            Self::RegToAcc(reg) => instructions::reg_to_acc(*reg, vm)?,
            Self::AccToReg(reg) => instructions::acc_to_reg(*reg, vm)?,

            Self::Return => instructions::return_()?,
            Self::ReturnAcc => instructions::return_acc(vm)?,
            Self::ReturnReg(reg) => instructions::return_reg(*reg, vm)?,
            Self::ReturnVar(name) => instructions::return_var(*name, vm)?,

            Self::ThrowAcc => instructions::throw_acc(vm)?,
            Self::ThrowReg(reg) => instructions::throw_reg(*reg, vm)?,
            Self::Throw(name) => instructions::throw(*name, vm)?,

            Self::ThisAcc => instructions::this_acc(vm)?,
            Self::ThisReg(reg) => instructions::this_reg(*reg, vm)?,
        }

        Ok(())
    }
}
