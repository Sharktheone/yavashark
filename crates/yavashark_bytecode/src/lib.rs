//! This crate contains bytecode definitions
//! it does not provide any way to execute or compile it.
//! 
//! 
//! 

mod writer;


type VarName = u32;
type ConstIdx = u32;
type Reg = u8;
type Lbl = u32;

#[repr(u16)]
enum Instruction {
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
    CallAcc,
    Jmp(Lbl),
    JmpRel(i32),
    JmpIf(VarName, Lbl),
    JmpIfAcc(Lbl),
    JmpIfRelAcc(i32),
    JmpIfNot(VarName, Lbl),
    JmpIfNotAcc(Lbl),
    JmpIfNotRelAcc(i32),
    JmpNull(VarName, Lbl),
    JmpNullAcc(Lbl),
    JmpNullRelAcc(i32),
    JmpUndef(VarName, Lbl),
    JmpUndefAcc(Lbl),
    JmpUndefRelAcc(i32),
    JmpNullUndef(VarName, Lbl),
    JmpNullUndefAcc(Lbl),
    JmpNullUndefRelAcc(i32),
    Str(VarName, ConstIdx),
    StrAcc(ConstIdx),
    LoadMemberAcc(VarName, VarName),
    LoadMemberReg(VarName, VarName, Reg),
    For,
    TypeOf(VarName, VarName),
    TypeOfAcc(VarName),
    InstanceOf(VarName, VarName),
    Return,
    Break,
    Continue,
    Throw,  
}