use crate::data::DataType;
use crate::instructions::Instruction;
use crate::{JmpAddr, JmpOffset};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum  Test {
    Always,
    Never,
    Cond(DataType),
    Not(DataType),
    Null(DataType),
    NotNull(DataType),
    Undefined(DataType),
    NotUndefined(DataType),
    Nullish(DataType),
    NotNullish(DataType),
    Eq(DataType, DataType),
    NotEq(DataType, DataType),
    StrictEq(DataType, DataType),
    StrictNotEq(DataType, DataType),
    Lt(DataType, DataType),
    LtEq(DataType, DataType),
    Gt(DataType, DataType),
    GtEq(DataType, DataType),
}


impl Test {
    #[must_use]
    pub fn get(self, addr: JmpAddr) -> Option<Instruction> {
        Some(match self {
            Self::Always => Instruction::jmp(addr),
            Self::Never => return None,
            Self::Cond(data) => Instruction::jmp_if(data, addr),
            Self::Not(data) => Instruction::jmp_if_not(data, addr),
            Self::Null(data) => Instruction::jmp_if_null(data, addr),
            Self::NotNull(data) => Instruction::jmp_if_not_null(data, addr),
            Self::Undefined(data) => Instruction::jmp_if_undefined(data, addr),
            Self::NotUndefined(data) => Instruction::jmp_if_not_undefined(data, addr),
            Self::Nullish(data) => Instruction::jmp_if_nullish(data, addr),
            Self::NotNullish(data) => Instruction::jmp_if_not_nullish(data, addr),
            Self::Eq(data1, data2) => Instruction::jmp_if_eq(data1, data2, addr),
            Self::NotEq(data1, data2) => Instruction::jmp_if_ne(data1, data2, addr),
            Self::StrictEq(data1, data2) => Instruction::jmp_if_strict_eq(data1, data2, addr),
            Self::StrictNotEq(data1, data2) => Instruction::jmp_if_strict_ne(data1, data2, addr),
            Self::Lt(data1, data2) => Instruction::jmp_if_lt(data1, data2, addr),
            Self::LtEq(data1, data2) => Instruction::jmp_if_lt_eq(data1, data2, addr),
            Self::Gt(data1, data2) => Instruction::jmp_if_gt(data1, data2, addr),
            Self::GtEq(data1, data2) => Instruction::jmp_if_gt_eq(data1, data2, addr),
        })

    }

    #[must_use]
    pub fn get_rel(self, addr: JmpOffset) -> Option<Instruction> {
        Some(match self {
            Self::Always => Instruction::jmp_rel(addr),
            Self::Never => return None,
            Self::Cond(data) => Instruction::jmp_if_rel(data, addr),
            Self::Not(data) => Instruction::jmp_if_not_rel(data, addr),
            Self::Null(data) => Instruction::jmp_if_null_rel(data, addr),
            Self::NotNull(data) => Instruction::jmp_if_not_null_rel(data, addr),
            Self::Undefined(data) => Instruction::jmp_if_undefined_rel(data, addr),
            Self::NotUndefined(data) => Instruction::jmp_if_not_undefined_rel(data, addr),
            Self::Nullish(data) => Instruction::jmp_if_nullish_rel(data, addr),
            Self::NotNullish(data) => Instruction::jmp_if_not_nullish_rel(data, addr),
            Self::Eq(data1, data2) => Instruction::jmp_if_eq_rel(data1, data2, addr),
            Self::NotEq(data1, data2) => Instruction::jmp_if_ne_rel(data1, data2, addr),
            Self::StrictEq(data1, data2) => Instruction::jmp_if_strict_eq_rel(data1, data2, addr),
            Self::StrictNotEq(data1, data2) => Instruction::jmp_if_strict_ne_rel(data1, data2, addr),
            Self::Lt(data1, data2) => Instruction::jmp_if_lt_rel(data1, data2, addr),
            Self::LtEq(data1, data2) => Instruction::jmp_if_lt_eq_rel(data1, data2, addr),
            Self::Gt(data1, data2) => Instruction::jmp_if_gt_rel(data1, data2, addr),
            Self::GtEq(data1, data2) => Instruction::jmp_if_gt_eq_rel(data1, data2, addr),
        })
    }
}