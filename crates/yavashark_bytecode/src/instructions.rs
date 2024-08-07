use crate::{ConstIdx, JmpOffset, Reg, VarName};

#[repr(u16)]
pub enum Instruction {
    Add(VarName, VarName),
    AddAcc(f64),
    AddStatic(VarName, f64),
    Sub(VarName, VarName),
    SubAcc(f64),
    SubStatic(VarName, f64),
    Div(VarName, VarName),
    DivAcc(f64),
    DivStatic(VarName, f64),
    Mul(VarName, VarName),
    MulAcc(f64),
    MulStatic(VarName, f64),
    LNot(VarName),
    LNotAcc,
    LOr(VarName, VarName),
    LOrAcc(Reg),
    LAnd(VarName, VarName),
    LAndAcc(Reg),
    LXor(VarName, VarName),
    LXorAcc(Reg),
    Dec(VarName),
    DecAcc,
    Inc(VarName),
    IncAcc,
    PushScope,
    PopScope,
    Call(VarName),
    CallReg(Reg),
    CallMember(VarName, VarName),
    CallMemberReg(Reg, VarName),
    CallMemberAcc(VarName),
    CallAcc,
    Jmp(JmpOffset),
    JmpRel(i32),
    JmpIf(VarName, JmpOffset),
    JmpIfAcc(JmpOffset),
    JmpIfRelAcc(i32),
    JmpIfNot(VarName, JmpOffset),
    JmpIfNotAcc(JmpOffset),
    JmpIfNotRelAcc(i32),
    JmpNull(VarName, JmpOffset),
    JmpNullAcc(JmpOffset),
    JmpNullRelAcc(i32),
    JmpUndef(VarName, JmpOffset),
    JmpUndefAcc(JmpOffset),
    JmpUndefRelAcc(i32),
    JmpNullUndef(VarName, JmpOffset),
    JmpNullUndefAcc(JmpOffset),
    JmpNullUndefRelAcc(i32),
    Str(VarName, ConstIdx),
    StrAcc(ConstIdx),
    Lda(VarName, ConstIdx),
    LdaAcc(ConstIdx),
    LoadMemberAcc(VarName, VarName),
    LoadMemberReg(VarName, VarName, Reg),
    LoadRegMember(Reg, VarName, Reg),
    LoadRegMemberAcc(Reg, VarName),
    LoadAccMember(VarName, Reg),
    LoadAccMemberAcc(VarName),
    LoadEnv(VarName),
    LoadEnvReg(VarName, Reg),
    For,
    TypeOf(VarName, VarName),
    TypeOfAcc(VarName),
    InstanceOf(VarName, VarName),
    PushConst(ConstIdx), // For stack operations
    PushReg(Reg),
    PushAcc,
    Pop,
    PopN(u32),
    PopToReg(Reg),
    PopToAcc,
    StackToReg(Reg),
    StackToAcc,
    StackIdxToReg(Reg, u32),
    StackIdxToAcc(u32),
    RegToAcc(Reg),
    AccToReg(Reg),
    Return,
    Break,
    Continue,
    Throw,
}
