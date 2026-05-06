use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct YSBigInt {
    sign: Sign,
    repr: BigIntRepr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Sign {
    Positive,
    Negative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BigIntRepr {
    Small(u64),
    Large([u64; 3]),
    Dynamic(Rc<[u64]>),
}

