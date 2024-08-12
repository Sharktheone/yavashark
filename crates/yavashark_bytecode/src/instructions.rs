use crate::{ConstIdx, JmpOffset, Reg, VarName};

#[repr(u16)]
pub enum Instruction {
    Add(VarName, VarName),
    AddAccReg(Reg),
    AddReg(Reg, Reg),
    
    Sub(VarName, VarName),
    SubAccReg(Reg),
    SubReg(Reg, Reg),
    
    Div(VarName, VarName),
    DivAccReg(Reg),
    DivReg(Reg, Reg),
    
    Mul(VarName, VarName),
    MulAccReg(Reg),
    MulReg(Reg, Reg),
    
    Mod(VarName, VarName),
    ModAccReg(Reg),
    ModReg(Reg, Reg),
    
    LNot(VarName),
    LNotAcc,
    
    LOr(VarName, VarName),
    LOrAcc(Reg),
    
    LAnd(VarName, VarName),
    LAndAcc(Reg),
    
    BitXor(VarName, VarName),
    BitXorAcc(Reg),
    BitXorReg(Reg, Reg),
    
    BitOr(VarName, VarName),
    BitOrAcc(Reg),
    BitOrReg(Reg, Reg),
    
    BitAnd(VarName, VarName),
    BitAndAcc(Reg),
    BitAndReg(Reg, Reg),
    
    Dec(VarName),
    DecAcc,
    DecReg(Reg),
    
    Inc(VarName),
    IncAcc,
    IncReg(Reg),
    
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
    ThrowAcc,
    ThrowReg,
    Throw(VarName),
}
