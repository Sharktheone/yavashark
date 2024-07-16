//! This crate contains bytecode definitions
//! it does not provide any way to execute or compile it.
//! 
//! 

#[repr(u16)]
enum Instruction {
    Add,
    Sub,
    Div,
    Mul,
    LNot,
    LOr,
    LAnd,
    LXor,
    Dec,
    Inc,
    PushScope,
    PopScope,
    Call,
    Jmp,
    JmpIf,
    JmpNull,
    JmpUndef,
    JmpNullUndef,
    Str,
    For,
    TypeOf,
    InstanceOf,
    Return,
    Break,
    Continue,
    Throw,  
}