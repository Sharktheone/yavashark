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
    
    Eq(VarName, VarName), // ==
    EqAcc(Reg),
    EqReg(Reg, Reg),
    
    NotEq(VarName, VarName), // !=
    NotEqAcc(Reg),
    NotEqReg(Reg, Reg),
    
    EqEq(VarName, VarName), // ===
    EqEqAcc(Reg),
    EqEqReg(Reg, Reg),
    
    NotEqEq(VarName, VarName), // !==
    NotEqEqAcc(Reg),
    NotEqEqReg(Reg, Reg),
    
    Lt(VarName, VarName),
    LtAcc(Reg),
    LtReg(Reg, Reg),
    
    LtEq(VarName, VarName),
    LtEqAcc(Reg),
    LtEqReg(Reg, Reg),
    
    Gt(VarName, VarName),
    GtAcc(Reg),
    GtReg(Reg, Reg),
    
    GtEq(VarName, VarName),
    GtEqAcc(Reg),
    GtEqReg(Reg, Reg),
    
    LShift(VarName, VarName),
    LShiftAcc(Reg),
    LShiftReg(Reg, Reg),
    
    RShift(VarName, VarName),
    RShiftAcc(Reg),
    RShiftReg(Reg, Reg),
    
    ZeroFillRShift(VarName, VarName),
    ZeroFillRShiftAcc(Reg),
    ZeroFillRShiftReg(Reg, Reg),
    
    In(VarName, VarName),
    InAcc(Reg),
    InReg(Reg, Reg),
    
    InstanceOf(VarName, VarName),
    InstanceOfAcc(Reg),
    InstanceOfReg(Reg, Reg),
    
    Exp(VarName, VarName),
    ExpAcc(Reg),
    ExpReg(Reg, Reg),
    
    NullishCoalescing(VarName, VarName),
    NullishCoalescingAcc(Reg),
    NullishCoalescingReg(Reg, Reg),
    
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
