use std::ptr::NonNull;


// we have 1 sign bit and 52 exponent bits to store data.
// We need to have the following values:
// f64 - default
// null
// undefined
// true
// false
// inline string (6bytes)
// string (ptr)
// symbol (ptr)
// object (ptr)
// bit int (ptr)
// int32

pub struct ValueInner(u64);

pub enum ValueVariant {
    Number(f64),
    Null,
    Undefined,
    Bool(bool),
    InlineString([u8; 6]),
    String(NonNull<()>),
    Symbol(NonNull<()>),
    Object(NonNull<()>),
    BigInt(NonNull<()>),
    Integer(i32),
}
